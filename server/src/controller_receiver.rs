use imgui::*;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::{ControllerInputData, AxisEvent};

#[derive(Debug, Clone)]
pub struct ReceivedInputEvent {
    pub timestamp: u64,
    pub controller_id: u32,
    pub event_type: String,
    pub details: String,
    pub delay_ms: u64,
}

pub struct ControllerReceiver {
    connected_clients: u32,
    total_events_received: u64,
    recent_events: VecDeque<ReceivedInputEvent>,
    max_events: usize,
    server_status: String,
    last_received_timestamp: u64,
    // Callback to send trigger events to virtual controller
    trigger_callback: Option<Box<dyn Fn(&str, f32) + Send + Sync>>,
}

impl ControllerReceiver {
    pub fn new() -> Self {
        Self {
            connected_clients: 0,
            total_events_received: 0,
            recent_events: VecDeque::new(),
            max_events: 100,
            server_status: "Starting...".to_string(),
            last_received_timestamp: 0,
            trigger_callback: None,
        }
    }

    pub fn update(&mut self) {
        // This would be called from the main loop
        // In a real implementation, you'd update the server status and client count here
        self.server_status = "Listening on 192.168.1.185:8080".to_string();
    }

    pub fn add_controller_event(&mut self, data: ControllerInputData) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let delay = if data.timestamp < current_time {
            current_time - data.timestamp
        } else {
            0
        };

        // Add button events
        for button_event in &data.button_events {
            let event = ReceivedInputEvent {
                timestamp: current_time,
                controller_id: data.controller_id,
                event_type: "Button".to_string(),
                details: format!("{} - {}", 
                    button_event.button, 
                    if button_event.pressed { "Pressed" } else { "Released" }),
                delay_ms: delay,
            };
            
            self.recent_events.push_back(event);
            self.total_events_received += 1;
            
            // Special handling for RT/LT digital button events
            if button_event.button.contains("RT [ID: 7]") || button_event.button.contains("Right Trigger") {
                log::info!("RT digital button event: {} -> {}", button_event.button, button_event.pressed);
                if let Some(ref callback) = self.trigger_callback {
                    callback("RT Axis", if button_event.pressed { 1.0 } else { 0.0 });
                }
            } else if button_event.button.contains("LT [ID: 6]") || button_event.button.contains("Left Trigger") {
                log::info!("LT digital button event: {} -> {}", button_event.button, button_event.pressed);
                if let Some(ref callback) = self.trigger_callback {
                    callback("LT Axis", if button_event.pressed { 1.0 } else { 0.0 });
                }
            }
        }

        // Add axis events
        for axis_event in &data.axis_events {
            let event = ReceivedInputEvent {
                timestamp: current_time,
                controller_id: data.controller_id,
                event_type: "Axis".to_string(),
                details: format!("{} - {:.3}", axis_event.axis, axis_event.value),
                delay_ms: delay,
            };
            
            self.recent_events.push_back(event);
            self.total_events_received += 1;
            
            // Special handling for RT/LT triggers - set to 100% when pressed
            if axis_event.axis.contains("RightZ") || axis_event.axis.contains("Right Trigger") {
                // RT (Right Trigger) pressed - set to 100%
                if axis_event.value > 0.1 {
                    log::info!("RT pressed - setting Xbox 360 RT to 100%");
                    if let Some(ref callback) = self.trigger_callback {
                        callback("RT Axis", 1.0); // Set to 100%
                    }
                } else {
                    log::info!("RT released - setting Xbox 360 RT to 0%");
                    if let Some(ref callback) = self.trigger_callback {
                        callback("RT Axis", 0.0); // Set to 0%
                    }
                }
            } else if axis_event.axis.contains("LeftZ") || axis_event.axis.contains("Left Trigger") {
                // LT (Left Trigger) pressed - set to 100%
                if axis_event.value > 0.1 {
                    log::info!("LT pressed - setting Xbox 360 LT to 100%");
                    if let Some(ref callback) = self.trigger_callback {
                        callback("LT Axis", 1.0); // Set to 100%
                    }
                } else {
                    log::info!("LT released - setting Xbox 360 LT to 0%");
                    if let Some(ref callback) = self.trigger_callback {
                        callback("LT Axis", 0.0); // Set to 0%
                    }
                }
            }
        }

        // Keep only the most recent events
        while self.recent_events.len() > self.max_events {
            self.recent_events.pop_front();
        }

        self.last_received_timestamp = current_time;
    }

    pub fn set_trigger_callback<F>(&mut self, callback: F) 
    where
        F: Fn(&str, f32) + Send + Sync + 'static,
    {
        self.trigger_callback = Some(Box::new(callback));
    }

    pub fn render(&mut self, ui: &Ui) {
        // Main menu bar
        ui.main_menu_bar(|| {
            ui.menu("View", || {
                ui.menu_item("Controller Events");
                ui.menu_item("Server Status");
            });
        });

        // Server Status Window
        ui.window("Server Status")
            .size([400.0, 200.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Steam Deck Controller Server");
                ui.separator();
                
                let status_color = if self.server_status.contains("Listening") {
                    [0.0, 1.0, 0.0, 1.0] // Green
                } else {
                    [1.0, 1.0, 0.0, 1.0] // Yellow
                };
                
                ui.text_colored(status_color, &format!("Status: {}", self.server_status));
                ui.text(&format!("Connected Clients: {}", self.connected_clients));
                ui.text(&format!("Total Events Received: {}", self.total_events_received));
                
                if self.last_received_timestamp > 0 {
                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    
                    let seconds_since_last = (current_time - self.last_received_timestamp) / 1000;
                    ui.text(&format!("Last Event: {}s ago", seconds_since_last));
                }
            });

        // Controller Events Window
        ui.window("Controller Events")
            .size([800.0, 600.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Real-time Controller Input from Steam Deck");
                ui.separator();
                
                if ui.button("Clear Events") {
                    self.recent_events.clear();
                }
                
                ui.same_line();
                ui.text(&format!("({} events)", self.recent_events.len()));
                
                ui.separator();
                
                // Table headers
                ui.columns(5, "events_table", true);
                ui.text("Timestamp");
                ui.next_column();
                ui.text("Controller");
                ui.next_column();
                ui.text("Type");
                ui.next_column();
                ui.text("Details");
                ui.next_column();
                ui.text("Delay (ms)");
                ui.next_column();
                ui.separator();
                
                // Event rows
                for event in self.recent_events.iter().rev() {
                    // Color code by delay
                    let delay_color = if event.delay_ms < 10 {
                        [0.0, 1.0, 0.0, 1.0] // Green - excellent
                    } else if event.delay_ms < 50 {
                        [1.0, 1.0, 0.0, 1.0] // Yellow - good
                    } else {
                        [1.0, 0.0, 0.0, 1.0] // Red - poor
                    };
                    
                    // Format timestamp
                    let timestamp_str = format!("{:.3}", (event.timestamp % 100000) as f64 / 1000.0);
                    ui.text(&timestamp_str);
                    ui.next_column();
                    
                    ui.text(&format!("{}", event.controller_id));
                    ui.next_column();
                    
                    // Color code by event type
                    let type_color = if event.event_type == "Button" {
                        [0.0, 0.8, 1.0, 1.0] // Blue for buttons
                    } else {
                        [1.0, 0.5, 0.0, 1.0] // Orange for axes
                    };
                    
                    ui.text_colored(type_color, &event.event_type);
                    ui.next_column();
                    
                    ui.text(&event.details);
                    ui.next_column();
                    
                    ui.text_colored(delay_color, &format!("{}", event.delay_ms));
                    ui.next_column();
                }
                
                ui.columns(1, "", false);
            });

        // Statistics Window
        ui.window("Performance Statistics")
            .size([400.0, 300.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Network Performance");
                ui.separator();
                
                if !self.recent_events.is_empty() {
                    let delays: Vec<u64> = self.recent_events.iter().map(|e| e.delay_ms).collect();
                    let avg_delay = delays.iter().sum::<u64>() as f64 / delays.len() as f64;
                    let min_delay = *delays.iter().min().unwrap_or(&0);
                    let max_delay = *delays.iter().max().unwrap_or(&0);
                    
                    ui.text(&format!("Average Delay: {:.2}ms", avg_delay));
                    ui.text(&format!("Min Delay: {}ms", min_delay));
                    ui.text(&format!("Max Delay: {}ms", max_delay));
                    
                    ui.separator();
                    
                    let button_events = self.recent_events.iter().filter(|e| e.event_type == "Button").count();
                    let axis_events = self.recent_events.iter().filter(|e| e.event_type == "Axis").count();
                    
                    ui.text(&format!("Button Events: {}", button_events));
                    ui.text(&format!("Axis Events: {}", axis_events));
                    
                    ui.separator();
                    
                    // Simple delay quality indicator
                    let quality = if avg_delay < 10.0 {
                        ("Excellent", [0.0, 1.0, 0.0, 1.0])
                    } else if avg_delay < 30.0 {
                        ("Good", [1.0, 1.0, 0.0, 1.0])
                    } else if avg_delay < 100.0 {
                        ("Fair", [1.0, 0.5, 0.0, 1.0])
                    } else {
                        ("Poor", [1.0, 0.0, 0.0, 1.0])
                    };
                    
                    ui.text("Connection Quality:");
                    ui.text_colored(quality.1, quality.0);
                } else {
                    ui.text("No events received yet...");
                    ui.text("Make sure the Steam Deck client is connected.");
                }
            });
    }
}
