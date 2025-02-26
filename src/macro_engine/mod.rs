mod combo;
mod event_processor;
mod timing;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque, HashSet};
use log::{debug, info, warn};

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
        if !is_down {
            // 处理键松开
            let mut blocked_keys = self.blocked_keys.lock().unwrap();
            if blocked_keys.contains(&key) {
                blocked_keys.remove(&key);
                return true; // 屏蔽按键传递
            }
            return false;
        }
        
        // 检查是否是英雄切换快捷键 (例如 Ctrl+Shift+F12)
        if self.check_hero_switch_hotkey(&key) {
            return true;
        }
        
        // 处理按键序列
        self.update_key_sequence(&key);
        
        // 检查当前活跃英雄的连招
        let hero_name = self.active_hero.lock().unwrap().clone();
        let hero_registry = self.hero_registry.lock().unwrap();
        
        if let Some(hero_config) = hero_registry.get_hero(&hero_name) {
            // 检查是否触发了任何连招
            for (combo_name, (trigger, action)) in &hero_config.combos {
                if self.check_combo_trigger(trigger) {
                    debug!("触发连招: {}", combo_name);
                    
                    // 修复：克隆整个KeyboardSimulator而不是引用
                    let mut simulator = KeyboardSimulator::new();
                    
                    // 执行连招动作
                    simulator.execute_actions(&action.keys);
                    
                    // 更新屏蔽按键
                    if !trigger.block_keys.is_empty() {
                        let mut blocked = self.blocked_keys.lock().unwrap();
                        for key in &trigger.block_keys {
                            blocked.insert(key.clone());
                        }
                    }
                    
                    return action.block_original;
                }
            }
        }
        
        false
    }
    
    fn update_key_sequence(&self, key: &Key) {
        let now = Instant::now();
        let mut sequence = self.key_sequence.lock().unwrap();
        
        // 添加新按键
        sequence.push_back((key.clone(), now));
        
        // 移除过时的按键
        while !sequence.is_empty() {
            if now.duration_since(sequence.front().unwrap().1) > self.sequence_window {
                sequence.pop_front();
            } else {
                break;
            }
        }
    }
    
    fn check_combo_trigger(&self, trigger: &ComboTrigger) -> bool {
        let sequence = self.key_sequence.lock().unwrap();
        
        // 检查序列长度
        if sequence.len() < trigger.sequence.len() {
            return false;
        }
        
        // 提取最近的N个按键进行比较
        let recent_keys: Vec<&Key> = sequence.iter()
            .map(|(k, _)| k)
            .rev()
            .take(trigger.sequence.len())
            .collect();
        
        // 反转以匹配原始顺序
        let recent_keys: Vec<&Key> = recent_keys.into_iter().rev().collect();
        
        // 检查按键序列是否匹配
        for (i, key) in trigger.sequence.iter().enumerate() {
            if key != recent_keys[i] {
                return false;
            }
        }
        
        // 检查时间窗口
        if let Some(window_ms) = trigger.time_window {
            let window = Duration::from_millis(window_ms);
            let first_key_time = sequence.iter()
                .rev()
                .take(trigger.sequence.len())
                .last()
                .unwrap()
                .1;
            let last_key_time = sequence.iter().last().unwrap().1;
            
            if last_key_time.duration_since(first_key_time) > window {
                return false;
            }
        }
        
        true
    }
    
    fn check_hero_switch_hotkey(&self, key: &Key) -> bool {
        // 这里仅为示例，检查是否按下了 Ctrl+Shift+F12
        // 实际应用中应当检查完整的组合键状态
        if let Key::F12 = key {
            // 通过UI切换英雄
            // ...
            return true;
        }
        
        // 检查是否是某个英雄的专属快捷键
        let hero_registry = self.hero_registry.lock().unwrap();
        for hero_name in hero_registry.get_hero_names() {
            if let Some(hero) = hero_registry.get_hero(&hero_name) {
                if let Some(hotkey) = &hero.hotkey {
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