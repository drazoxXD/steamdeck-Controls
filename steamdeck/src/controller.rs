use anyhow::Result;
use gilrs::{Gilrs, Event, EventType, Button, Axis};
use log::{info, warn, error};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::protocol::*;

pub struct ControllerManager {
    sender: mpsc::Sender<ControllerState>,
    gilrs: Gilrs,
    controller_states: HashMap<usize, ControllerState>,
}

impl ControllerManager {
    pub fn new(sender: mpsc::Sender<ControllerState>) -> Self {
        let gilrs = Gilrs::new().expect("Failed to initialize controller subsystem");
        info!("Controller subsystem initialized");
        
        Self {
            sender,
            gilrs,
            controller_states: HashMap::new(),
        }
    }

    pub async fn run(&mut self, controller_list: Arc<Mutex<Vec<ControllerInfo>>>) {
        info!("Starting controller manager");
        
        // Initial controller scan
        self.scan_controllers(&controller_list).await;

        loop {
            // Process controller events
            while let Some(Event { id, event, time: _ }) = self.gilrs.next_event() {
                match event {
                    EventType::ButtonPressed(button, _) => {
                        self.handle_button_press(id, button).await;
                    }
                    EventType::ButtonReleased(button, _) => {
                        self.handle_button_release(id, button).await;
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        self.handle_axis_change(id, axis, value).await;
                    }
                    EventType::Connected => {
                        info!("Controller {} connected", id);
                        self.scan_controllers(&controller_list).await;
                    }
                    EventType::Disconnected => {
                        info!("Controller {} disconnected", id);
                        self.controller_states.remove(&id);
                        self.scan_controllers(&controller_list).await;
                    }
                    _ => {}
                }
            }

            // Update controller states
            self.update_controller_states().await;

            sleep(Duration::from_millis(16)).await; // ~60 FPS
        }
    }

    async fn scan_controllers(&mut self, controller_list: &Arc<Mutex<Vec<ControllerInfo>>>) {
        let mut controllers = Vec::new();
        
        for (id, gamepad) in self.gilrs.gamepads() {
            let info = ControllerInfo {
                name: gamepad.name().to_string(),
                uuid: format!("{:016x}", id), // Use ID as UUID for now
                vendor_id: 0x28de, // Steam Controller vendor ID
                product_id: 0x1102, // Steam Controller product ID
                connected: gamepad.is_connected(),
            };
            
            controllers.push(info.clone());
            info!("Found controller: {} (ID: {})", info.name, id);
            
            // Initialize controller state if not exists
            if !self.controller_states.contains_key(&id) {
                self.controller_states.insert(id, ControllerState::default());
            }
        }

        // Update the shared controller list
        if let Ok(mut list) = controller_list.lock() {
            *list = controllers;
        }
    }

    async fn handle_button_press(&mut self, id: usize, button: Button) {
        if let Some(state) = self.controller_states.get_mut(&id) {
            match button {
                Button::South => state.button_a = true,
                Button::East => state.button_b = true,
                Button::West => state.button_x = true,
                Button::North => state.button_y = true,
                Button::LeftTrigger => state.button_lb = true,
                Button::RightTrigger => state.button_rb = true,
                Button::Select => state.button_back = true,
                Button::Start => state.button_start = true,
                Button::Mode => state.button_guide = true,
                Button::LeftThumb => state.button_l3 = true,
                Button::RightThumb => state.button_r3 = true,
                Button::DPadUp => state.dpad_up = true,
                Button::DPadDown => state.dpad_down = true,
                Button::DPadLeft => state.dpad_left = true,
                Button::DPadRight => state.dpad_right = true,
                _ => {}
            }
            
            state.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
        }
    }

    async fn handle_button_release(&mut self, id: usize, button: Button) {
        if let Some(state) = self.controller_states.get_mut(&id) {
            match button {
                Button::South => state.button_a = false,
                Button::East => state.button_b = false,
                Button::West => state.button_x = false,
                Button::North => state.button_y = false,
                Button::LeftTrigger => state.button_lb = false,
                Button::RightTrigger => state.button_rb = false,
                Button::Select => state.button_back = false,
                Button::Start => state.button_start = false,
                Button::Mode => state.button_guide = false,
                Button::LeftThumb => state.button_l3 = false,
                Button::RightThumb => state.button_r3 = false,
                Button::DPadUp => state.dpad_up = false,
                Button::DPadDown => state.dpad_down = false,
                Button::DPadLeft => state.dpad_left = false,
                Button::DPadRight => state.dpad_right = false,
                _ => {}
            }
            
            state.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
        }
    }

    async fn handle_axis_change(&mut self, id: usize, axis: Axis, value: f32) {
        if let Some(state) = self.controller_states.get_mut(&id) {
            match axis {
                Axis::LeftStickX => state.left_stick_x = value,
                Axis::LeftStickY => state.left_stick_y = value,
                Axis::RightStickX => state.right_stick_x = value,
                Axis::RightStickY => state.right_stick_y = value,
                Axis::LeftZ => state.left_trigger = (value + 1.0) / 2.0, // Convert from -1..1 to 0..1
                Axis::RightZ => state.right_trigger = (value + 1.0) / 2.0,
                _ => {}
            }
            
            state.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;
        }
    }

    async fn update_controller_states(&self) {
        // Send the first controller's state (assuming we want to use the first connected controller)
        if let Some((_, state)) = self.controller_states.iter().next() {
            if let Err(e) = self.sender.send(state.clone()).await {
                error!("Failed to send controller state: {}", e);
            }
        }
    }
}
