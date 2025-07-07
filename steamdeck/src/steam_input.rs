use anyhow::Result;
use std::collections::HashMap;
use gilrs::{GamepadId, Button, Axis};

pub struct SteamInputManager {
    initialized: bool,
    digital_actions: HashMap<String, bool>,
    analog_actions: HashMap<String, (f32, f32)>,
    controller_handles: Vec<GamepadId>,
    action_sets: Vec<u64>,
    // Map gilrs buttons/axes to Steam Input actions
    button_mappings: HashMap<Button, String>,
    axis_mappings: HashMap<Axis, String>,
}

impl SteamInputManager {
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            initialized: false,
            digital_actions: HashMap::new(),
            analog_actions: HashMap::new(),
            controller_handles: Vec::new(),
            action_sets: Vec::new(),
            button_mappings: HashMap::new(),
            axis_mappings: HashMap::new(),
        };

        manager.initialize()?;
        Ok(manager)
    }

    fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        
        // Initialize digital actions with button names, IDs, and action descriptions
        self.digital_actions.insert("A (South) [ID: 0] - Jump".to_string(), false);
        self.digital_actions.insert("B (East) [ID: 1] - Fire".to_string(), false);
        self.digital_actions.insert("X (West) [ID: 2] - Reload".to_string(), false);
        self.digital_actions.insert("Y (North) [ID: 3] - Menu".to_string(), false);
        self.digital_actions.insert("LB [ID: 4] - Use".to_string(), false);
        self.digital_actions.insert("RB [ID: 5] - Sprint".to_string(), false);
        self.digital_actions.insert("LT [ID: 6] - Aim".to_string(), false);
        self.digital_actions.insert("RT [ID: 7] - Fire".to_string(), false);
        self.digital_actions.insert("LSB [ID: 8] - Sprint".to_string(), false);
        self.digital_actions.insert("RSB [ID: 9] - Crouch".to_string(), false);
        self.digital_actions.insert("Start [ID: 10] - Menu".to_string(), false);
        self.digital_actions.insert("Select [ID: 11] - Map".to_string(), false);
        self.digital_actions.insert("D-Pad Up [ID: 12] - Quick Action 1".to_string(), false);
        self.digital_actions.insert("D-Pad Down [ID: 13] - Quick Action 2".to_string(), false);
        self.digital_actions.insert("D-Pad Left [ID: 14] - Quick Action 3".to_string(), false);
        self.digital_actions.insert("D-Pad Right [ID: 15] - Quick Action 4".to_string(), false);
        
        // Initialize analog actions with proper names (all start at 0,0)
        self.analog_actions.insert("Left Stick - Move".to_string(), (0.0, 0.0));
        self.analog_actions.insert("Right Stick - Look".to_string(), (0.0, 0.0));
        self.analog_actions.insert("Left Trigger - Aim".to_string(), (0.0, 0.0));
        self.analog_actions.insert("Right Trigger - Fire".to_string(), (0.0, 0.0));
        
        // Set up button mappings (map gamepad buttons to Steam Input action names)
        // Note: In gilrs, LeftTrigger/RightTrigger are bumpers (LB/RB), LeftTrigger2/RightTrigger2 are triggers (LT/RT)
        self.button_mappings.insert(Button::South, "A (South) [ID: 0] - Jump".to_string());
        self.button_mappings.insert(Button::East, "B (East) [ID: 1] - Fire".to_string());
        self.button_mappings.insert(Button::West, "X (West) [ID: 2] - Reload".to_string());
        self.button_mappings.insert(Button::North, "Y (North) [ID: 3] - Menu".to_string());
        self.button_mappings.insert(Button::LeftTrigger, "LB [ID: 4] - Use".to_string());        // Bumper
        self.button_mappings.insert(Button::RightTrigger, "RB [ID: 5] - Sprint".to_string());    // Bumper
        self.button_mappings.insert(Button::LeftTrigger2, "LT [ID: 6] - Aim".to_string());       // Trigger
        self.button_mappings.insert(Button::RightTrigger2, "RT [ID: 7] - Fire".to_string());     // Trigger
        self.button_mappings.insert(Button::LeftThumb, "LSB [ID: 8] - Sprint".to_string());
        self.button_mappings.insert(Button::RightThumb, "RSB [ID: 9] - Crouch".to_string());
        self.button_mappings.insert(Button::Start, "Start [ID: 10] - Menu".to_string());
        self.button_mappings.insert(Button::Select, "Select [ID: 11] - Map".to_string());
        self.button_mappings.insert(Button::DPadUp, "D-Pad Up [ID: 12] - Quick Action 1".to_string());
        self.button_mappings.insert(Button::DPadDown, "D-Pad Down [ID: 13] - Quick Action 2".to_string());
        self.button_mappings.insert(Button::DPadLeft, "D-Pad Left [ID: 14] - Quick Action 3".to_string());
        self.button_mappings.insert(Button::DPadRight, "D-Pad Right [ID: 15] - Quick Action 4".to_string());
        
        // Set up axis mappings
        self.axis_mappings.insert(Axis::LeftStickX, "Left Stick - Move".to_string());
        self.axis_mappings.insert(Axis::LeftStickY, "Left Stick - Move".to_string());
        self.axis_mappings.insert(Axis::RightStickX, "Right Stick - Look".to_string());
        self.axis_mappings.insert(Axis::RightStickY, "Right Stick - Look".to_string());
        self.axis_mappings.insert(Axis::LeftZ, "Left Trigger - Aim".to_string());
        self.axis_mappings.insert(Axis::RightZ, "Right Trigger - Fire".to_string());
        
        log::info!("Steam Input initialized with real controller mappings");
        Ok(())
    }

    pub fn update(&mut self) {
        // This method is now called from the main loop, but the actual updates
        // happen via the update_from_controller_input method
    }

    // New method to update Steam Input based on real controller input
    pub fn update_from_controller_input(&mut self, controller_id: GamepadId, button: Option<(Button, bool)>, axis: Option<(Axis, f32)>) {
        if !self.initialized {
            return;
        }

        // Add controller to our list if not already present
        if !self.controller_handles.contains(&controller_id) {
            self.controller_handles.push(controller_id);
        }

        // Handle button input
        if let Some((btn, pressed)) = button {
            if let Some(action_name) = self.button_mappings.get(&btn) {
                self.digital_actions.insert(action_name.clone(), pressed);
                log::debug!("Button {:?} -> Action '{}': {}", btn, action_name, pressed);
            }
        }

        // Handle axis input
        if let Some((ax, value)) = axis {
            if let Some(action_name) = self.axis_mappings.get(&ax) {
                let current = self.analog_actions.get(action_name).copied().unwrap_or((0.0, 0.0));
                
                match ax {
                    Axis::LeftStickX | Axis::RightStickX => {
                        // X axis for sticks
                        self.analog_actions.insert(action_name.clone(), (value, current.1));
                    }
                    Axis::LeftStickY | Axis::RightStickY => {
                        // Y axis for sticks (invert for typical game controls)
                        self.analog_actions.insert(action_name.clone(), (current.0, -value));
                    }
                    Axis::LeftZ => {
                        // Left trigger (L2) - store as X component for "Left Trigger - Aim"
                        self.analog_actions.insert(action_name.clone(), (value, 0.0));
                        
                        // Also update the digital action for LT button press
                        let pressed = value > 0.1; // Threshold for digital press
                        self.digital_actions.insert("LT [ID: 6] - Aim".to_string(), pressed);
                    }
                    Axis::RightZ => {
                        // Right trigger (R2) - store as X component for "Right Trigger - Fire"
                        self.analog_actions.insert(action_name.clone(), (value, 0.0));
                        
                        // Also update the digital action for RT button press
                        let pressed = value > 0.1; // Threshold for digital press
                        self.digital_actions.insert("RT [ID: 7] - Fire".to_string(), pressed);
                    }
                    _ => {
                        // Other axes - treat as X component
                        self.analog_actions.insert(action_name.clone(), (value, current.1));
                    }
                }
                log::debug!("Axis {:?} -> Action '{}': {:.3}", ax, action_name, value);
            }
        }
    }

    pub fn remove_controller(&mut self, controller_id: GamepadId) {
        self.controller_handles.retain(|&id| id != controller_id);
        
        // Reset all actions if no controllers are connected
        if self.controller_handles.is_empty() {
            // Reset all digital actions to false
            for (_, pressed) in self.digital_actions.iter_mut() {
                *pressed = false;
            }
            // Reset all analog actions to (0,0)
            for (_, (x, y)) in self.analog_actions.iter_mut() {
                *x = 0.0;
                *y = 0.0;
            }
            log::info!("All controllers disconnected - resetting all actions");
        }
    }

    pub fn get_digital_actions(&self) -> HashMap<String, bool> {
        self.digital_actions.clone()
    }

    pub fn get_analog_actions(&self) -> HashMap<String, (f32, f32)> {
        self.analog_actions.clone()
    }

    pub fn get_controller_count(&self) -> usize {
        self.controller_handles.len()
    }

    pub fn get_connected_controllers(&self) -> Vec<String> {
        let mut controllers = Vec::new();
        for (i, &controller_id) in self.controller_handles.iter().enumerate() {
            controllers.push(format!("Controller {} (ID: {})", i + 1, controller_id));
        }
        
        // Add Steam Deck controller if we detect it
        if self.is_steam_deck() {
            controllers.push("Steam Deck Built-in Controller".to_string());
        }
        
        controllers
    }

    fn is_steam_deck(&self) -> bool {
        // Check if we're running on Steam Deck
        std::env::var("SteamDeck").is_ok() || 
        std::env::var("STEAM_DECK").is_ok() ||
        self.check_steam_deck_hardware()
    }

    fn check_steam_deck_hardware(&self) -> bool {
        // Simple check for Steam Deck - look for specific hardware indicators
        // This is a basic implementation, you could make it more sophisticated
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            hostname.to_lowercase().contains("steamdeck")
        } else {
            false
        }
    }

    pub fn shutdown(&mut self) {
        if self.initialized {
            self.initialized = false;
            log::info!("Steam Input shutdown");
        }
    }

    pub fn get_button_mappings(&self) -> HashMap<Button, String> {
        self.button_mappings.clone()
    }

    pub fn get_axis_mappings(&self) -> HashMap<Axis, String> {
        self.axis_mappings.clone()
    }

    pub fn get_action_for_button(&self, button: Button) -> Option<String> {
        self.button_mappings.get(&button).cloned()
    }

    pub fn get_action_for_axis(&self, axis: Axis) -> Option<String> {
        self.axis_mappings.get(&axis).cloned()
    }

    pub fn get_debug_json(&self) -> String {
        use serde_json::json;
        
        let debug_data = json!({
            "initialized": self.initialized,
            "controller_count": self.controller_handles.len(),
            "connected_controllers": self.get_connected_controllers(),
            "digital_actions": self.digital_actions,
            "analog_actions": self.analog_actions,
            "button_mappings": self.button_mappings.iter().map(|(button, action)| {
                (format!("{:?}", button), action.clone())
            }).collect::<std::collections::HashMap<_, _>>(),
            "axis_mappings": self.axis_mappings.iter().map(|(axis, action)| {
                (format!("{:?}", axis), action.clone())
            }).collect::<std::collections::HashMap<_, _>>(),
            "raw_controller_ids": self.controller_handles.iter().map(|id| format!("{:?}", id)).collect::<Vec<_>>(),
            "axis_info": {
                "LeftStickX": "ID 1 - Left stick horizontal",
                "LeftStickY": "ID 2 - Left stick vertical", 
                "LeftZ": "ID 3 - Left trigger (L2) analog",
                "RightStickX": "ID 4 - Right stick horizontal",
                "RightStickY": "ID 5 - Right stick vertical",
                "RightZ": "ID 6 - Right trigger (R2) analog",
                "DPadX": "ID 7 - D-pad horizontal",
                "DPadY": "ID 8 - D-pad vertical"
            }
        });
        
        serde_json::to_string_pretty(&debug_data).unwrap_or_else(|_| "Failed to serialize debug data".to_string())
    }
}

impl Drop for SteamInputManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
