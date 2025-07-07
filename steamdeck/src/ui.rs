#[cfg(feature = "gui")]
use eframe::egui;
#[cfg(feature = "gui")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "gui")]
use crate::protocol::*;

#[cfg(feature = "gui")]
pub struct SteamDeckUI {
    controller_state: Arc<Mutex<ControllerState>>,
    controller_list: Arc<Mutex<Vec<ControllerInfo>>>,
}

#[cfg(feature = "gui")]
impl SteamDeckUI {
    pub fn new(
        controller_state: Arc<Mutex<ControllerState>>,
        controller_list: Arc<Mutex<Vec<ControllerInfo>>>,
    ) -> Self {
        Self {
            controller_state,
            controller_list,
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for SteamDeckUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SteamDeck Controller Client");
            
            ui.separator();
            
            // Controller List
            ui.heading("Available Controllers");
            if let Ok(controllers) = self.controller_list.lock() {
                if controllers.is_empty() {
                    ui.label("No controllers detected");
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
            
            // Controller State Debug
            ui.heading("Controller State Debug");
            if let Ok(state) = self.controller_state.lock() {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Left Stick:");
                        ui.label(format!("X: {:.2}", state.left_stick_x));
                        ui.label(format!("Y: {:.2}", state.left_stick_y));
                        
                        ui.separator();
                        
                        ui.label("Right Stick:");
                        ui.label(format!("X: {:.2}", state.right_stick_x));
                        ui.label(format!("Y: {:.2}", state.right_stick_y));
                        
                        ui.separator();
                        
                        ui.label("Triggers:");
                        ui.label(format!("Left: {:.2}", state.left_trigger));
                        ui.label(format!("Right: {:.2}", state.right_trigger));
                    });
                    
                    ui.vertical(|ui| {
                        ui.label("Face Buttons:");
                        ui.horizontal(|ui| {
                            if state.button_a { ui.colored_label(egui::Color32::GREEN, "A"); } else { ui.label("A"); }
                            if state.button_b { ui.colored_label(egui::Color32::GREEN, "B"); } else { ui.label("B"); }
                            if state.button_x { ui.colored_label(egui::Color32::GREEN, "X"); } else { ui.label("X"); }
                            if state.button_y { ui.colored_label(egui::Color32::GREEN, "Y"); } else { ui.label("Y"); }
                        });
                        
                        ui.label("Shoulder Buttons:");
                        ui.horizontal(|ui| {
                            if state.button_lb { ui.colored_label(egui::Color32::GREEN, "LB"); } else { ui.label("LB"); }
                            if state.button_rb { ui.colored_label(egui::Color32::GREEN, "RB"); } else { ui.label("RB"); }
                            if state.button_l3 { ui.colored_label(egui::Color32::GREEN, "L3"); } else { ui.label("L3"); }
                            if state.button_r3 { ui.colored_label(egui::Color32::GREEN, "R3"); } else { ui.label("R3"); }
                        });
                        
                        ui.label("System Buttons:");
                        ui.horizontal(|ui| {
                            if state.button_start { ui.colored_label(egui::Color32::GREEN, "Start"); } else { ui.label("Start"); }
                            if state.button_back { ui.colored_label(egui::Color32::GREEN, "Back"); } else { ui.label("Back"); }
                            if state.button_guide { ui.colored_label(egui::Color32::GREEN, "Guide"); } else { ui.label("Guide"); }
                        });
                        
                        ui.label("D-Pad:");
                        ui.horizontal(|ui| {
                            if state.dpad_up { ui.colored_label(egui::Color32::GREEN, "↑"); } else { ui.label("↑"); }
                            if state.dpad_down { ui.colored_label(egui::Color32::GREEN, "↓"); } else { ui.label("↓"); }
                            if state.dpad_left { ui.colored_label(egui::Color32::GREEN, "←"); } else { ui.label("←"); }
                            if state.dpad_right { ui.colored_label(egui::Color32::GREEN, "→"); } else { ui.label("→"); }
                        });
                    });
                });
                
                ui.separator();
                ui.label(format!("Timestamp: {}", state.timestamp));
            }
        });
    }
}
