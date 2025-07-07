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
    show_raw_input: bool,
    show_steam_input: bool,
    show_controller_mapping: bool,
    show_input_history: bool,
    input_history: Vec<String>,
    max_history_size: usize,
    steam_input_data: Option<SteamInputData>,
}

#[derive(Debug, Clone)]
pub struct SteamInputData {
    pub digital_actions: HashMap<String, bool>,
    pub analog_actions: HashMap<String, (f32, f32)>,
    pub controller_count: usize,
    pub connected_controllers: Vec<String>,
}

impl ControllerDebugUI {
    pub fn new() -> Self {
        Self {
            controllers: HashMap::new(),
            show_raw_input: true,
            show_steam_input: true,
            show_controller_mapping: true,
            show_input_history: true,
            input_history: Vec::new(),
            max_history_size: 100,
            steam_input_data: None,
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
        });
    }

    fn add_to_history(&mut self, message: String) {
        self.input_history.push(format!("[{}] {}", 
            chrono::Utc::now().format("%H:%M:%S%.3f"), 
            message));
        
        if self.input_history.len() > self.max_history_size {
            self.input_history.remove(0);
        }
    }

    pub fn render(&mut self, ui: &Ui) {
        // Main menu bar
        ui.main_menu_bar(|| {
            ui.menu("View", || {
                ui.checkbox("Raw Input", &mut self.show_raw_input);
                ui.checkbox("Steam Input", &mut self.show_steam_input);
                ui.checkbox("Controller Mapping", &mut self.show_controller_mapping);
                ui.checkbox("Input History", &mut self.show_input_history);
            });
        });

        // Controller overview
        Window::new("Controller Overview")
            .size([400.0, 300.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(format!("Connected Controllers: {}", self.controllers.len()));
                ui.separator();
                
                for (id, controller) in &self.controllers {
                    let color = if controller.connected {
                        [0.0, 1.0, 0.0, 1.0] // Green for connected
                    } else {
                        [1.0, 0.0, 0.0, 1.0] // Red for disconnected
                    };
                    
                    ui.text_colored(color, format!("Controller {}: {}", id, controller.name));
                    ui.text(format!("  Last Activity: {:.2}s ago", 
                        controller.last_activity.elapsed().as_secs_f32()));
                    ui.text(format!("  Buttons: {} pressed", 
                        controller.buttons.values().filter(|&&v| v).count()));
                    ui.text(format!("  Axes: {} active", 
                        controller.axes.values().filter(|&&v| v.abs() > 0.1).count()));
                }
            });

        // Raw input display
        if self.show_raw_input {
            Window::new("Raw Controller Input")
                .size([500.0, 400.0], Condition::FirstUseEver)
                .build(ui, || {
                    for (id, controller) in &self.controllers {
                        if ui.collapsing_header(format!("Controller {} - {}", id, controller.name), TreeNodeFlags::empty()) {
                            ui.text("Buttons:");
                            ui.indent();
                            for (button, &pressed) in &controller.buttons {
                                let color = if pressed {
                                    [0.0, 1.0, 0.0, 1.0]
                                } else {
                                    [0.7, 0.7, 0.7, 1.0]
                                };
                                ui.text_colored(color, format!("{:?}: {}", button, pressed));
                            }
                            ui.unindent();
                            
                            ui.text("Axes:");
                            ui.indent();
                            for (axis, &value) in &controller.axes {
                                let color = if value.abs() > 0.1 {
                                    [1.0, 1.0, 0.0, 1.0]
                                } else {
                                    [0.7, 0.7, 0.7, 1.0]
                                };
                                ui.text_colored(color, format!("{:?}: {:.3}", axis, value));
                                
                                // Visual bar for axis values
                                let progress = (value + 1.0) / 2.0; // Convert from -1..1 to 0..1
                                ui.progress_bar(progress)
                                    .size([200.0, 0.0])
                                    .overlay_text(format!("{:.3}", value));
                            }
                            ui.unindent();
                        }
                    }
                });
        }

        // Steam Input display
        if self.show_steam_input {
            Window::new("Steam Input")
                .size([500.0, 400.0], Condition::FirstUseEver)
                .build(ui, || {
                    if let Some(ref steam_data) = self.steam_input_data {
                        ui.text(format!("Steam Controllers: {}", steam_data.controller_count));
                        ui.separator();
                        
                        if ui.collapsing_header("Connected Controllers", TreeNodeFlags::DefaultOpen) {
                            for controller in &steam_data.connected_controllers {
                                ui.text(format!("• {}", controller));
                            }
                        }
                        
                        if ui.collapsing_header("Digital Actions", TreeNodeFlags::DefaultOpen) {
                            for (action, &active) in &steam_data.digital_actions {
                                let color = if active {
                                    [0.0, 1.0, 0.0, 1.0]
                                } else {
                                    [0.7, 0.7, 0.7, 1.0]
                                };
                                ui.text_colored(color, format!("{}: {}", action, active));
                            }
                        }
                        
                        if ui.collapsing_header("Analog Actions", TreeNodeFlags::DefaultOpen) {
                            for (action, &(x, y)) in &steam_data.analog_actions {
                                let magnitude = (x * x + y * y).sqrt();
                                let color = if magnitude > 0.1 {
                                    [1.0, 1.0, 0.0, 1.0]
                                } else {
                                    [0.7, 0.7, 0.7, 1.0]
                                };
                                ui.text_colored(color, format!("{}: ({:.3}, {:.3})", action, x, y));
                                
                                // 2D visualization for analog sticks
                                let draw_list = ui.get_window_draw_list();
                                let canvas_pos = ui.cursor_screen_pos();
                                let canvas_size = [100.0, 100.0];
                                let center = [canvas_pos[0] + canvas_size[0] / 2.0, canvas_pos[1] + canvas_size[1] / 2.0];
                                
                                // Draw background circle
                                draw_list.add_circle(center, 40.0, [0.3, 0.3, 0.3, 1.0])
                                    .filled(true)
                                    .build();
                                
                                // Draw stick position
                                let stick_pos = [
                                    center[0] + x * 35.0,
                                    center[1] - y * 35.0, // Invert Y for screen coordinates
                                ];
                                draw_list.add_circle(stick_pos, 5.0, [1.0, 0.0, 0.0, 1.0])
                                    .filled(true)
                                    .build();
                                
                                ui.dummy(canvas_size);
                            }
                        }
                    } else {
                        ui.text("Steam Input not available");
                        ui.text("Make sure Steam is running and the game is launched through Steam");
                    }
                });
        }

        // Controller mapping display
        if self.show_controller_mapping {
            Window::new("Controller Mapping")
                .size([400.0, 300.0], Condition::FirstUseEver)
                .build(ui, || {
                    ui.text("Button Mapping:");
                    ui.separator();
                    
                    let mappings = [
                        ("A/Cross", "Jump/Confirm"),
                        ("B/Circle", "Back/Cancel"),
                        ("X/Square", "Reload/Interact"),
                        ("Y/Triangle", "Menu/Map"),
                        ("Left Bumper", "Aim/Block"),
                        ("Right Bumper", "Shoot/Attack"),
                        ("Left Trigger", "Aim Down Sights"),
                        ("Right Trigger", "Fire"),
                        ("D-Pad", "Quick Actions"),
                        ("Left Stick", "Movement"),
                        ("Right Stick", "Camera/Look"),
                        ("Left Stick Click", "Sprint/Run"),
                        ("Right Stick Click", "Melee/Crouch"),
                        ("Start/Options", "Pause Menu"),
                        ("Back/Share", "Map/Inventory"),
                    ];
                    
                    for (button, action) in mappings {
                        ui.text(format!("{}: {}", button, action));
                    }
                });
        }

        // Input history
        if self.show_input_history {
            Window::new("Input History")
                .size([600.0, 300.0], Condition::FirstUseEver)
                .build(ui, || {
                    if ui.button("Clear History") {
                        self.input_history.clear();
                    }
                    ui.same_line();
                    ui.text(format!("({}/{} entries)", self.input_history.len(), self.max_history_size));
                    
                    ui.separator();
                    
                    ui.child_window("history_scroll")
                        .size([0.0, 0.0])
                        .build(|| {
                            for entry in &self.input_history {
                                ui.text(entry);
                            }
                        });
                });
        }
    }
}
