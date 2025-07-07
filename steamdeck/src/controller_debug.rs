use gilrs::{GamepadId, EventType, Button, Axis};
use imgui::*;
use std::collections::HashMap;
use std::time::Instant;
use crate::steam_input::SteamInputManager;

#[derive(Debug, Clone)]
pub struct ControllerState {
    pub id: GamepadId,
    pub name: String,
    pub buttons: HashMap<Button, bool>,
    pub axes: HashMap<Axis, f32>,
    pub last_activity: Instant,
    pub connected: bool,
}

impl ControllerState {
    pub fn new(id: GamepadId, name: String) -> Self {
        Self {
            id,
            name,
            buttons: HashMap::new(),
            axes: HashMap::new(),
            last_activity: Instant::now(),
            connected: true,
        }
    }

    pub fn update_button(&mut self, button: Button, pressed: bool) {
        self.buttons.insert(button, pressed);
        self.last_activity = Instant::now();
    }

    pub fn update_axis(&mut self, axis: Axis, value: f32) {
        self.axes.insert(axis, value);
        self.last_activity = Instant::now();
    }
}

pub struct ControllerDebugUI {
    controllers: HashMap<GamepadId, ControllerState>,
    show_steam_input: bool,
    show_network_options: bool,
    input_history: Vec<String>,
    max_history_size: usize,
    steam_input_data: Option<SteamInputData>,
    // Network settings
    network_enabled: bool,
    server_ip: String,
    server_port: i32,
    connection_status: String,
}

#[derive(Debug, Clone)]
pub struct SteamInputData {
    pub digital_actions: HashMap<String, bool>,
    pub analog_actions: HashMap<String, (f32, f32)>,
    pub controller_count: usize,
    pub connected_controllers: Vec<String>,
    pub button_mappings: HashMap<Button, String>,
    pub axis_mappings: HashMap<Axis, String>,
}

impl ControllerDebugUI {
    pub fn new() -> Self {
        Self {
            controllers: HashMap::new(),
            show_steam_input: true,
            show_network_options: true,
            input_history: Vec::new(),
            max_history_size: 100,
            steam_input_data: None,
            network_enabled: false,
            server_ip: "192.168.1.185".to_string(),
            server_port: 8080,
            connection_status: "Disconnected".to_string(),
        }
    }

    fn get_button_display_name(button: &Button) -> &'static str {
        match button {
            Button::South => "A (South)",
            Button::East => "B (East)", 
            Button::North => "Y (North)",
            Button::West => "X (West)",
            Button::LeftTrigger => "LT (LeftTrigger)",
            Button::LeftTrigger2 => "LT2 (LeftTrigger2)",
            Button::RightTrigger => "RT (RightTrigger)",
            Button::RightTrigger2 => "RT2 (RightTrigger2)",
            Button::Select => "Select/View",
            Button::Start => "Start/Menu",
            Button::Mode => "Guide/Steam",
            Button::LeftThumb => "LSB (LeftThumb)",
            Button::RightThumb => "RSB (RightThumb)",
            Button::DPadUp => "D-Pad Up",
            Button::DPadDown => "D-Pad Down",
            Button::DPadLeft => "D-Pad Left",
            Button::DPadRight => "D-Pad Right",
            Button::Unknown => "Unknown",
            _ => "Other",
        }
    }

    fn get_axis_display_name(axis: &Axis) -> &'static str {
        match axis {
            Axis::LeftStickX => "Left Stick X",
            Axis::LeftStickY => "Left Stick Y",
            Axis::LeftZ => "Left Z (L2)",
            Axis::RightStickX => "Right Stick X",
            Axis::RightStickY => "Right Stick Y",
            Axis::RightZ => "Right Z (R2)",
            Axis::DPadX => "D-Pad X",
            Axis::DPadY => "D-Pad Y",
            Axis::Unknown => "Unknown",
            _ => "Other",
        }
    }

    fn get_button_name(button: Button) -> &'static str {
        match button {
            Button::South => "A (South)",
            Button::East => "B (East)", 
            Button::North => "Y (North)",
            Button::West => "X (West)",
            Button::LeftTrigger => "LB (LeftTrigger)",
            Button::LeftTrigger2 => "LT (LeftTrigger2)",
            Button::RightTrigger => "RB (RightTrigger)",
            Button::RightTrigger2 => "RT (RightTrigger2)",
            Button::Select => "Select/View",
            Button::Start => "Start/Menu",
            Button::Mode => "Guide/Steam",
            Button::LeftThumb => "LSB (LeftThumb)",
            Button::RightThumb => "RSB (RightThumb)",
            Button::DPadUp => "D-Pad Up",
            Button::DPadDown => "D-Pad Down",
            Button::DPadLeft => "D-Pad Left",
            Button::DPadRight => "D-Pad Right",
            Button::Unknown => "Unknown",
            _ => "Other",
        }
    }

    fn get_axis_name(axis: Axis) -> &'static str {
        match axis {
            Axis::LeftStickX => "Left Stick X",
            Axis::LeftStickY => "Left Stick Y",
            Axis::LeftZ => "LT Axis (LeftZ)",
            Axis::RightStickX => "Right Stick X",
            Axis::RightStickY => "Right Stick Y",
            Axis::RightZ => "RT Axis (RightZ)",
            Axis::DPadX => "D-Pad X",
            Axis::DPadY => "D-Pad Y",
            Axis::Unknown => "Unknown",
            _ => "Other",
        }
    }

    pub fn handle_gilrs_event(&mut self, id: GamepadId, event: EventType, _time: f64) {
        match event {
            EventType::Connected => {
                let name = format!("Controller {}", id);
                self.controllers.insert(id, ControllerState::new(id, name.clone()));
                self.add_to_history(format!("Controller {} connected: {}", id, name));
            }
            EventType::Disconnected => {
                if let Some(controller) = self.controllers.get_mut(&id) {
                    controller.connected = false;
                    self.add_to_history(format!("Controller {} disconnected", id));
                }
            }
            EventType::ButtonPressed(button, _) => {
                if let Some(controller) = self.controllers.get_mut(&id) {
                    controller.update_button(button, true);
                    self.add_to_history(format!("Controller {} - Button {:?} pressed", id, button));
                }
            }
            EventType::ButtonReleased(button, _) => {
                if let Some(controller) = self.controllers.get_mut(&id) {
                    controller.update_button(button, false);
                    self.add_to_history(format!("Controller {} - Button {:?} released", id, button));
                }
            }
            EventType::AxisChanged(axis, value, _) => {
                if let Some(controller) = self.controllers.get_mut(&id) {
                    controller.update_axis(axis, value);
                    self.add_to_history(format!("Controller {} - Axis {:?}: {:.3}", id, axis, value));
                }
            }
            EventType::ButtonChanged(button, value, _) => {
                if let Some(controller) = self.controllers.get_mut(&id) {
                    controller.update_button(button, value > 0.5);
                    self.add_to_history(format!("Controller {} - Button {:?} changed: {:.3}", id, button, value));
                }
            }
            _ => {}
        }
    }

    pub fn update_steam_input(&mut self, steam_input: &SteamInputManager) {
        self.steam_input_data = Some(SteamInputData {
            digital_actions: steam_input.get_digital_actions(),
            analog_actions: steam_input.get_analog_actions(),
            controller_count: steam_input.get_controller_count(),
            connected_controllers: steam_input.get_connected_controllers(),
            button_mappings: steam_input.get_button_mappings(),
            axis_mappings: steam_input.get_axis_mappings(),
        });
    }

    fn add_to_history(&mut self, message: String) {
        self.input_history.push(format!("[{}] {}", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(), 
            message));
        
        if self.input_history.len() > self.max_history_size {
            self.input_history.remove(0);
        }
    }

    pub fn render(&mut self, ui: &Ui) {
        // Main menu bar
        ui.main_menu_bar(|| {
            ui.menu("View", || {
                ui.checkbox("Steam Input", &mut self.show_steam_input);
                ui.checkbox("Network Options", &mut self.show_network_options);
            });
        });

        // Network Options
        if self.show_network_options {
            ui.window("Network Controller Streaming")
                .size([400.0, 300.0], Condition::FirstUseEver)
                .build(|| {
                    ui.text("Stream controller input to PC over network");
                    ui.separator();
                    
                    ui.text("Server Settings:");
                    ui.input_text("Server IP", &mut self.server_ip).build();
                    ui.input_int("Port", &mut self.server_port).build();
                    
                    ui.separator();
                    
                    if ui.button("Connect") && !self.network_enabled {
                        self.network_enabled = true;
                        self.connection_status = "Connecting...".to_string();
                        // TODO: Implement network connection
                    }
                    
                    ui.same_line();
                    
                    if ui.button("Disconnect") && self.network_enabled {
                        self.network_enabled = false;
                        self.connection_status = "Disconnected".to_string();
                        // TODO: Implement network disconnection
                    }
                    
                    ui.separator();
                    
                    let status_color = match self.connection_status.as_str() {
                        "Connected" => [0.0, 1.0, 0.0, 1.0],
                        "Connecting..." => [1.0, 1.0, 0.0, 1.0],
                        _ => [1.0, 0.0, 0.0, 1.0],
                    };
                    
                    ui.text_colored(status_color, &format!("Status: {}", self.connection_status));
                    
                    if self.network_enabled {
                        ui.text(&format!("Streaming to: {}:{}", self.server_ip, self.server_port));
                        ui.text(&format!("Connected Controllers: {}", self.controllers.len()));
                    }
                });
        }

        // Steam Input display
        if self.show_steam_input {
            ui.window("Steam Input")
                .size([500.0, 400.0], Condition::FirstUseEver)
                .build(|| {
                    if let Some(ref steam_data) = self.steam_input_data {
                        ui.text(&format!("Steam Controllers: {}", steam_data.controller_count));
                        ui.separator();
                        
                        if ui.collapsing_header("Connected Controllers", TreeNodeFlags::empty()) {
                            for controller in &steam_data.connected_controllers {
                                ui.text(&format!("• {}", controller));
                            }
                        }
                        
                        if ui.collapsing_header("Digital Actions", TreeNodeFlags::empty()) {
                            ui.text("Current Active Actions:");
                            ui.separator();
                            
                            // Group actions by type for better display
                            let mut face_buttons = Vec::new();
                            let mut shoulder_buttons = Vec::new();
                            let mut trigger_buttons = Vec::new();
                            let mut stick_buttons = Vec::new();
                            let mut dpad_buttons = Vec::new();
                            let mut menu_buttons = Vec::new();
                            
                            for (action, &active) in &steam_data.digital_actions {
                                if action.contains("A (South)") || action.contains("B (East)") || 
                                   action.contains("X (West)") || action.contains("Y (North)") {
                                    face_buttons.push((action, active));
                                } else if action.contains("LB") || action.contains("RB") {
                                    shoulder_buttons.push((action, active));
                                } else if action.contains("LT") || action.contains("RT") {
                                    trigger_buttons.push((action, active));
                                } else if action.contains("LSB") || action.contains("RSB") {
                                    stick_buttons.push((action, active));
                                } else if action.contains("D-Pad") {
                                    dpad_buttons.push((action, active));
                                } else if action.contains("Start") || action.contains("Select") {
                                    menu_buttons.push((action, active));
                                }
                            }
                            
                            // Display grouped actions
                            if !face_buttons.is_empty() {
                                ui.text("Face Buttons:");
                                for (action, active) in face_buttons {
                                    let color = if active { [0.0, 1.0, 0.0, 1.0] } else { [0.7, 0.7, 0.7, 1.0] };
                                    ui.text_colored(color, &format!("  {}: {}", action, active));
                                }
                            }
                            
                            if !shoulder_buttons.is_empty() {
                                ui.text("Shoulder Buttons:");
                                for (action, active) in shoulder_buttons {
                                    let color = if active { [0.0, 1.0, 0.0, 1.0] } else { [0.7, 0.7, 0.7, 1.0] };
                                    ui.text_colored(color, &format!("  {}: {}", action, active));
                                }
                            }
                            
                            if !trigger_buttons.is_empty() {
                                ui.text("Triggers:");
                                for (action, active) in trigger_buttons {
                                    let color = if active { [0.0, 1.0, 0.0, 1.0] } else { [0.7, 0.7, 0.7, 1.0] };
                                    ui.text_colored(color, &format!("  {}: {}", action, active));
                                }
                            }
                            
                            if !stick_buttons.is_empty() {
                                ui.text("Stick Buttons:");
                                for (action, active) in stick_buttons {
                                    let color = if active { [0.0, 1.0, 0.0, 1.0] } else { [0.7, 0.7, 0.7, 1.0] };
                                    ui.text_colored(color, &format!("  {}: {}", action, active));
                                }
                            }
                            
                            if !dpad_buttons.is_empty() {
                                ui.text("D-Pad:");
                                for (action, active) in dpad_buttons {
                                    let color = if active { [0.0, 1.0, 0.0, 1.0] } else { [0.7, 0.7, 0.7, 1.0] };
                                    ui.text_colored(color, &format!("  {}: {}", action, active));
                                }
                            }
                            
                            if !menu_buttons.is_empty() {
                                ui.text("Menu Buttons:");
                                for (action, active) in menu_buttons {
                                    let color = if active { [0.0, 1.0, 0.0, 1.0] } else { [0.7, 0.7, 0.7, 1.0] };
                                    ui.text_colored(color, &format!("  {}: {}", action, active));
                                }
                            }
                        }
                        
                        if ui.collapsing_header("Analog Actions", TreeNodeFlags::empty()) {
                            for (action, &(x, y)) in &steam_data.analog_actions {
                                let magnitude = (x * x + y * y).sqrt();
                                let color = if magnitude > 0.1 {
                                    [1.0, 1.0, 0.0, 1.0]
                                } else {
                                    [0.7, 0.7, 0.7, 1.0]
                                };
                                ui.text_colored(color, &format!("{}: ({:.3}, {:.3})", action, x, y));
                            }
                        }
                    } else {
                        ui.text("Steam Input not available");
                        ui.text("Make sure Steam is running and the game is launched through Steam");
                    }
                });
        }
    }

    pub fn get_network_settings(&self) -> (bool, String, i32) {
        (self.network_enabled, self.server_ip.clone(), self.server_port)
    }

    pub fn set_connection_status(&mut self, status: String) {
        self.connection_status = status;
    }

    pub fn set_network_enabled(&mut self, enabled: bool) {
        self.network_enabled = enabled;
    }

    pub fn should_connect_network(&self) -> bool {
        // This would be set when the connect button is pressed
        // For now, we'll implement a simple check
        false
    }

    pub fn should_disconnect_network(&self) -> bool {
        // This would be set when the disconnect button is pressed
        false
    }
}
