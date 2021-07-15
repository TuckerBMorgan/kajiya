#![allow(dead_code)]

use glam::Vec2;
use std::{collections::HashMap, hash::Hash};
use winit::event::WindowEvent;
pub use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Clone)]
pub struct KeyState {
    pub ticks: u32,
}

#[derive(Default, Clone)]
pub struct KeyboardState {
    keys_down: HashMap<VirtualKeyCode, KeyState>,
}

impl KeyboardState {
    pub fn is_down(&self, key: VirtualKeyCode) -> bool {
        self.get_down(key).is_some()
    }

    pub fn was_just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.get_down(key).map(|s| s.ticks == 1).unwrap_or_default()
    }

    pub fn get_down(&self, key: VirtualKeyCode) -> Option<&KeyState> {
        self.keys_down.get(&key)
    }

    pub fn update(&mut self, events: &[WindowEvent]) {
        for event in events {
            if let WindowEvent::KeyboardInput { input, .. } = event {
                if let Some(vk) = input.virtual_keycode {
                    if input.state == ElementState::Pressed {
                        self.keys_down.entry(vk).or_insert(KeyState { ticks: 0 });
                    } else {
                        self.keys_down.remove(&vk);
                    }
                }
            }
        }

        for ks in self.keys_down.values_mut() {
            ks.ticks += 1;
        }
    }
}

#[derive(Clone, Copy)]
pub struct MouseState {
    pub pos: Vec2,
    pub delta: Vec2,
    pub button_mask: u32,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            delta: Vec2::ZERO,
            button_mask: 0,
        }
    }
}

impl MouseState {
    pub fn update(&mut self, events: &[WindowEvent]) {
        let prev_pos = self.pos;

        for event in events {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.pos = Vec2::new(position.x as f32, position.y as f32);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let button_id = match button {
                        winit::event::MouseButton::Left => 0,
                        winit::event::MouseButton::Middle => 1,
                        winit::event::MouseButton::Right => 2,
                        _ => 0,
                    };

                    if let ElementState::Pressed = state {
                        self.button_mask |= 1 << button_id;
                    } else {
                        self.button_mask &= !(1 << button_id);
                    }
                }
                _ => (),
            }
        }

        self.delta = self.pos - prev_pos;
    }
}

pub type InputAxis = &'static str;

pub struct KeyMap {
    axis: InputAxis,
    multiplier: f32,
    activation_time: f32,
}

impl KeyMap {
    pub fn new(axis: InputAxis, multiplier: f32) -> Self {
        Self {
            axis,
            multiplier,
            activation_time: 0.15,
        }
    }

    pub fn activation_time(mut self, value: f32) -> Self {
        self.activation_time = value;
        self
    }
}

struct KeyMapState {
    map: KeyMap,
    activation: f32,
}

pub struct KeyboardMap {
    bindings: Vec<(VirtualKeyCode, KeyMapState)>,
}

impl KeyboardMap {
    pub fn new() -> Self {
        Self {
            bindings: Default::default(),
        }
    }

    pub fn bind(mut self, key: VirtualKeyCode, map: KeyMap) -> Self {
        self.bindings.push((
            key,
            KeyMapState {
                map,
                activation: 0.0,
            },
        ));
        self
    }

    pub fn map(&mut self, keyboard: &KeyboardState, dt: f32) -> HashMap<InputAxis, f32> {
        let mut result: HashMap<InputAxis, f32> = HashMap::new();

        for (vk, s) in &mut self.bindings {
            if s.map.activation_time > 1e-10 {
                let change = if keyboard.is_down(*vk) { dt } else { -dt };
                s.activation = (s.activation + change / s.map.activation_time).clamp(0.0, 1.0);
            } else {
                if keyboard.is_down(*vk) {
                    s.activation = 1.0;
                } else {
                    s.activation = 0.0;
                }
            }

            *result.entry(s.map.axis).or_default() += s.activation * s.map.multiplier;
        }

        for value in result.values_mut() {
            *value = value.clamp(-1.0, 1.0);
        }

        result
    }
}
