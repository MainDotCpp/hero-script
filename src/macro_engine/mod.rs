mod combo;
mod event_processor;
mod timing;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::{VecDeque, HashSet};
use log::{debug, info, warn, error};

use crate::config::hero::{Key, ComboTrigger, ComboAction, KeyAction};
use crate::keyboard::simulator::KeyboardSimulator;
use crate::heroes::HeroRegistry;

use self::event_processor::KeyEventProcessor;
use self::timing::TimingWindow;

pub struct MacroEngine {
    keyboard_simulator: Arc<KeyboardSimulator>,
    hero_registry: Arc<Mutex<HeroRegistry>>,
    active_hero: Arc<Mutex<String>>,
    event_processor: Mutex<KeyEventProcessor>,
    key_sequence: Mutex<VecDeque<(Key, Instant)>>,
    blocked_keys: Mutex<HashSet<Key>>,
    sequence_window: Duration,
}

impl MacroEngine {
    pub fn new(
        keyboard_simulator: Arc<KeyboardSimulator>,
        hero_registry: Arc<Mutex<HeroRegistry>>,
        active_hero: Arc<Mutex<String>>,
    ) -> Self {
        let sequence_window = Duration::from_millis(1000); // 默认1秒的按键序列窗口
        
        Self {
            keyboard_simulator,
            hero_registry,
            active_hero,
            event_processor: Mutex::new(KeyEventProcessor::new()),
            key_sequence: Mutex::new(VecDeque::new()),
            blocked_keys: Mutex::new(HashSet::new()),
            sequence_window,
        }
    }
    
    pub fn process_key_event(&self, key: Key, is_down: bool) -> bool {
        info!("宏引擎处理按键: {:?}, 状态: {}", key, if is_down { "按下" } else { "释放" });
        
        if !is_down {
            // 处理键松开
            let mut blocked_keys = self.blocked_keys.lock().unwrap();
            if blocked_keys.contains(&key) {
                info!("屏蔽已释放的按键: {:?}", key);
                blocked_keys.remove(&key);
                return true;
            }
            return false;
        }
        
        // 更新事件处理器中的按键状态
        {
            let mut processor = self.event_processor.lock().unwrap();
            processor.set_key_state(key.clone(), true);
            debug!("更新按键状态: {:?} = {}", key, true);
        }
        
        // 检查是否是英雄切换快捷键
        if self.check_hero_switch_hotkey(&key) {
            info!("触发英雄切换快捷键: {:?}", key);
            return true;
        }
        
        // 处理按键序列
        self.update_key_sequence(&key);
        let current_sequence = self.get_current_sequence();
        info!("当前按键序列: {:?}", current_sequence);
        
        // 检查当前活跃英雄的连招
        let hero_name = self.active_hero.lock().unwrap().clone();
        info!("检查英雄 [{}] 的连招", hero_name);
        let hero_registry = self.hero_registry.lock().unwrap();
        
        if let Some(hero_config) = hero_registry.get_hero(&hero_name) {
            debug!("找到英雄 [{}] 的配置，检查连招", hero_name);
            // 检查是否触发了任何连招
            for (combo_name, (trigger, action)) in &hero_config.combos {
                debug!("检查连招: {} 触发条件: {:?}", combo_name, trigger.sequence);
                
                if self.check_combo_trigger(trigger) {
                    info!("触发连招: {}", combo_name);
                    
                    // 创建新的键盘模拟器实例
                    let mut simulator = KeyboardSimulator::new();
                    
                    // 执行连招动作
                    debug!("执行连招动作: {:?}", action.keys);
                    simulator.execute_actions(&action.keys);
                    
                    // 更新屏蔽按键
                    if !trigger.block_keys.is_empty() {
                        debug!("设置屏蔽按键: {:?}", trigger.block_keys);
                        let mut blocked = self.blocked_keys.lock().unwrap();
                        for key in &trigger.block_keys {
                            blocked.insert(key.clone());
                        }
                    }
                    
                    return action.block_original;
                }
            }
        } else {
            warn!("未找到英雄 [{}] 的配置", hero_name);
        }
        
        false
    }
    
    fn check_combo_trigger(&self, trigger: &ComboTrigger) -> bool {
        // 检查是否满足时间窗口条件
        if let Some(time_window) = trigger.time_window {
            debug!("检查时间窗口连招，窗口: {}ms", time_window);
            // 从按键序列中检查是否按下了触发按键序列
            let sequence = self.key_sequence.lock().unwrap();
            
            // 检查序列是否为空
            if sequence.len() < trigger.sequence.len() {
                debug!("按键序列长度不足");
                return false;
            }
            
            // 创建一个映射来跟踪每个按键的最后出现时间
            let mut key_times = std::collections::HashMap::new();
            for (key, time) in sequence.iter() {
                key_times.insert(key, *time);
            }
            
            // 检查所有触发键是否都存在
            for key in &trigger.sequence {
                if !key_times.contains_key(key) {
                    debug!("按键 {:?} 不在序列中", key);
                    return false;
                }
            }
            
            // 检查按键之间的时间差是否在时间窗口内
            let first_key_time = *key_times.get(&trigger.sequence[0]).unwrap();
            for i in 1..trigger.sequence.len() {
                let key_time = *key_times.get(&trigger.sequence[i]).unwrap();
                let diff = key_time.duration_since(first_key_time).as_millis();
                debug!("按键 {:?} 和 {:?} 时间差: {}ms", trigger.sequence[0], trigger.sequence[i], diff);
                
                if diff > time_window as u128 {
                    debug!("时间差超过窗口");
                    return false;
                }
            }
            
            debug!("满足时间窗口条件");
            return true;
        } else {
            // 检查是否同时按下了所有触发键
            debug!("检查同时按键连招");
            let processor = self.event_processor.lock().unwrap();
            let result = processor.are_keys_down(&trigger.sequence);
            debug!("同时按键检查结果: {}", result);
            return result;
        }
    }
    
    fn update_key_sequence(&self, key: &Key) {
        let now = Instant::now();
        let mut sequence = self.key_sequence.lock().unwrap();
        
        // 添加新按键到序列
        sequence.push_back((key.clone(), now));
        
        // 移除过期的按键
        while !sequence.is_empty() {
            if let Some((_, time)) = sequence.front() {
                if now.duration_since(*time) > self.sequence_window {
                    sequence.pop_front();
                } else {
                    break;
                }
            }
        }
    }
    
    fn get_current_sequence(&self) -> Vec<Key> {
        let sequence = self.key_sequence.lock().unwrap();
        sequence.iter().map(|(key, _)| key.clone()).collect()
    }
    
    fn check_hero_switch_hotkey(&self, key: &Key) -> bool {
        let hero_registry = self.hero_registry.lock().unwrap();
        let hero_names = hero_registry.get_hero_names();
        
        for hero_name in &hero_names {
            if let Some(hero_config) = hero_registry.get_hero(hero_name) {
                if let Some(hotkey) = &hero_config.hotkey {
                    if hotkey.len() == 1 && &hotkey[0] == key {
                        let mut active = self.active_hero.lock().unwrap();
                        *active = hero_name.clone();
                        info!("切换到英雄: {}", hero_name);
                        return true;
                    }
                }
            }
        }
        
        false
    }
} 