use eframe::egui;
use std::sync::{Arc, Mutex};
use crate::protocol::*;

pub struct WindowsUI {
    controller_state: Arc<Mutex<ControllerState>>,
    controller_list: Arc<Mutex<Vec<ControllerInfo>>>,
    received_inputs: Arc<Mutex<Vec<String>>>,
    connection_status: String,
}

impl WindowsUI {
    pub fn new(
        controller_state: Arc<Mutex<ControllerState>>,
        controller_list: Arc<Mutex<Vec<ControllerInfo>>>,
        received_inputs: Arc<Mutex<Vec<String>>>,
    ) -> Self {
        Self {
            controller_state,
            controller_list,
            received_inputs,
            connection_status: "Waiting for connection...".to_string(),
        }
    }
}

impl eframe::App for WindowsUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Windows Controller Host - SteamDeck Receiver");
            
            ui.separator();
            
            // Connection Status
            ui.horizontal(|ui| {
                ui.label("Connection Status:");
                ui.colored_label(egui::Color32::from_rgb(0, 150, 0), &self.connection_status);
            });
            
            ui.separator();
            
            // Virtual Controller Status
            ui.heading("Virtual Controller Status");
            ui.horizontal(|ui| {
                ui.label("🎮 Virtual Xbox 360 Controller:");
                ui.colored_label(egui::Color32::from_rgb(0, 150, 0), "Connected as 'SteamDeck Controller'");
            });
            
            ui.separator();
            
            // Remote Controllers
            ui.heading("Remote Controllers (SteamDeck)");
            if let Ok(controllers) = self.controller_list.lock() {
                if controllers.is_empty() {
                    ui.label("No remote controllers detected");
                } else {
                    for controller in controllers.iter() {
                        ui.horizontal(|ui| {
                            let status = if controller.connected { "🟢" } else { "🔴" };
                            ui.label(format!("{} {}", status, controller.name));
                            ui.label(format!("VID: {:04X}, PID: {:04X}", controller.vendor_id, controller.product_id));
                        });
                    }
                }
            }
            
            ui.separator();
            
            // Controller State Visualization
            ui.heading("Received Controller State");
            if let Ok(state) = self.controller_state.lock() {
                ui.horizontal(|ui| {
                    // Left column - Sticks and Triggers
                    ui.vertical(|ui| {
                        ui.label("Left Stick:");
                        ui.horizontal(|ui| {
                            ui.label(format!("X: {:.2}", state.left_stick_x));
                            ui.add(egui::ProgressBar::new((state.left_stick_x + 1.0) / 2.0).text("X"));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Y: {:.2}", state.left_stick_y));
                            ui.add(egui::ProgressBar::new((state.left_stick_y + 1.0) / 2.0).text("Y"));
                        });
                        
                        ui.separator();
                        
                        ui.label("Right Stick:");
                        ui.horizontal(|ui| {
                            ui.label(format!("X: {:.2}", state.right_stick_x));
                            ui.add(egui::ProgressBar::new((state.right_stick_x + 1.0) / 2.0).text("X"));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("Y: {:.2}", state.right_stick_y));
                            ui.add(egui::ProgressBar::new((state.right_stick_y + 1.0) / 2.0).text("Y"));
                        });
                        
                        ui.separator();
                        
                        ui.label("Triggers:");
                        ui.horizontal(|ui| {
                            ui.label(format!("L: {:.2}", state.left_trigger));
                            ui.add(egui::ProgressBar::new(state.left_trigger).text("LT"));
                        });
                        ui.horizontal(|ui| {
                            ui.label(format!("R: {:.2}", state.right_trigger));
                            ui.add(egui::ProgressBar::new(state.right_trigger).text("RT"));
                        });
                    });
                    
                    ui.separator();
                    
                    // Right column - Buttons
                    ui.vertical(|ui| {
                        ui.label("Face Buttons:");
                        ui.horizontal(|ui| {
                            if state.button_a { 
                                ui.colored_label(egui::Color32::GREEN, "🅰"); 
                            } else { 
                                ui.label("🅰"); 
                            }
                            if state.button_b { 
                                ui.colored_label(egui::Color32::GREEN, "🅱"); 
                            } else { 
                                ui.label("🅱"); 
                            }
                            if state.button_x { 
                                ui.colored_label(egui::Color32::GREEN, "🅧"); 
                            } else { 
                                ui.label("🅧"); 
                            }
                            if state.button_y { 
                                ui.colored_label(egui::Color32::GREEN, "🅨"); 
                            } else { 
                                ui.label("🅨"); 
                            }
                        });
                        
                        ui.separator();
                        
                        ui.label("Shoulder Buttons:");
                        ui.horizontal(|ui| {
                            if state.button_lb { 
                                ui.colored_label(egui::Color32::GREEN, "LB"); 
                            } else { 
                                ui.label("LB"); 
                            }
                            if state.button_rb { 
                                ui.colored_label(egui::Color32::GREEN, "RB"); 
                            } else { 
                                ui.label("RB"); 
                            }
                            if state.button_l3 { 
                                ui.colored_label(egui::Color32::GREEN, "L3"); 
                            } else { 
                                ui.label("L3"); 
                            }
                            if state.button_r3 { 
                                ui.colored_label(egui::Color32::GREEN, "R3"); 
                            } else { 
                                ui.label("R3"); 
                            }
                        });
                        
                        ui.separator();
                        
                        ui.label("System Buttons:");
                        ui.horizontal(|ui| {
                            if state.button_start { 
                                ui.colored_label(egui::Color32::GREEN, "START"); 
                            } else { 
                                ui.label("START"); 
                            }
                            if state.button_back { 
                                ui.colored_label(egui::Color32::GREEN, "BACK"); 
                            } else { 
                                ui.label("BACK"); 
                            }
                            if state.button_guide { 
                                ui.colored_label(egui::Color32::GREEN, "GUIDE"); 
                            } else { 
                                ui.label("GUIDE"); 
                            }
                        });
                        
                        ui.separator();
                        
                        ui.label("D-Pad:");
                        ui.horizontal(|ui| {
                            if state.dpad_up { 
                                ui.colored_label(egui::Color32::GREEN, "↑"); 
                            } else { 
                                ui.label("↑"); 
                            }
                            if state.dpad_down { 
                                ui.colored_label(egui::Color32::GREEN, "↓"); 
                            } else { 
                                ui.label("↓"); 
                            }
                            if state.dpad_left { 
                                ui.colored_label(egui::Color32::GREEN, "←"); 
                            } else { 
                                ui.label("←"); 
                            }
                            if state.dpad_right { 
                                ui.colored_label(egui::Color32::GREEN, "→"); 
                            } else { 
                                ui.label("→"); 
                            }
                        });
                    });
                });
                
                ui.separator();
                ui.label(format!("Last Update: {}", state.timestamp));
            }
            
            ui.separator();
            
            // Input Log
            ui.heading("Input Activity Log");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    if let Ok(inputs) = self.received_inputs.lock() {
                        if inputs.is_empty() {
                            ui.label("No input activity yet...");
                        } else {
                            for input in inputs.iter().rev() {
                                ui.label(input);
                            }
                        }
                    }
                });
        });
    }
}
