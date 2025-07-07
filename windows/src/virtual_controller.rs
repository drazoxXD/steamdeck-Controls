use anyhow::Result;
use log::info;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::protocol::*;

pub struct VirtualControllerManager {
    // We'll use a simple approach since vigem-client might not be available
    // In a real implementation, you'd use vigem-client or similar
    connected: bool,
}

impl VirtualControllerManager {
    pub fn new() -> Result<Self> {
        info!("Initializing virtual controller manager");
        
        // In a real implementation, you'd initialize ViGEm client here
        // For now, we'll simulate it
        Ok(Self {
            connected: false,
        })
    }

    pub async fn run(
        &self,
        mut controller_rx: mpsc::Receiver<ControllerState>,
        controller_state: Arc<Mutex<ControllerState>>,
        received_inputs: Arc<Mutex<Vec<String>>>,
    ) {
        info!("Starting virtual controller manager");

        // Simulate connecting a virtual Xbox controller
        info!("Virtual Xbox 360 Controller connected as 'SteamDeck Controller'");

        let mut last_state = ControllerState::default();

        loop {
            tokio::select! {
                // Receive controller state updates
                state = controller_rx.recv() => {
                    if let Some(state) = state {
                        // Update shared state
                        if let Ok(mut current_state) = controller_state.lock() {
                            *current_state = state.clone();
                        }

                        // Log input changes for debug
                        self.log_input_changes(&last_state, &state, &received_inputs).await;
                        
                        // Send to virtual controller
                        self.update_virtual_controller(&state).await;
                        
                        last_state = state;
                    }
                }

                // Regular update cycle
                _ = sleep(Duration::from_millis(16)) => {
                    // Regular maintenance if needed
                }
            }
        }
    }

    async fn log_input_changes(
        &self,
        old_state: &ControllerState,
        new_state: &ControllerState,
        received_inputs: &Arc<Mutex<Vec<String>>>,
    ) {
        let mut changes = Vec::new();

        // Check button changes
        if old_state.button_a != new_state.button_a {
            changes.push(format!("Button A: {}", if new_state.button_a { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_b != new_state.button_b {
            changes.push(format!("Button B: {}", if new_state.button_b { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_x != new_state.button_x {
            changes.push(format!("Button X: {}", if new_state.button_x { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_y != new_state.button_y {
            changes.push(format!("Button Y: {}", if new_state.button_y { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_lb != new_state.button_lb {
            changes.push(format!("Left Bumper: {}", if new_state.button_lb { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_rb != new_state.button_rb {
            changes.push(format!("Right Bumper: {}", if new_state.button_rb { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_start != new_state.button_start {
            changes.push(format!("Start: {}", if new_state.button_start { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_back != new_state.button_back {
            changes.push(format!("Back: {}", if new_state.button_back { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_guide != new_state.button_guide {
            changes.push(format!("Guide: {}", if new_state.button_guide { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_l3 != new_state.button_l3 {
            changes.push(format!("Left Stick: {}", if new_state.button_l3 { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.button_r3 != new_state.button_r3 {
            changes.push(format!("Right Stick: {}", if new_state.button_r3 { "PRESSED" } else { "RELEASED" }));
        }

        // Check D-pad changes
        if old_state.dpad_up != new_state.dpad_up {
            changes.push(format!("D-Pad Up: {}", if new_state.dpad_up { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.dpad_down != new_state.dpad_down {
            changes.push(format!("D-Pad Down: {}", if new_state.dpad_down { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.dpad_left != new_state.dpad_left {
            changes.push(format!("D-Pad Left: {}", if new_state.dpad_left { "PRESSED" } else { "RELEASED" }));
        }
        if old_state.dpad_right != new_state.dpad_right {
            changes.push(format!("D-Pad Right: {}", if new_state.dpad_right { "PRESSED" } else { "RELEASED" }));
        }

        // Check analog changes (with threshold to avoid spam)
        let threshold = 0.1;
        if (old_state.left_stick_x - new_state.left_stick_x).abs() > threshold {
            changes.push(format!("Left Stick X: {:.2}", new_state.left_stick_x));
        }
        if (old_state.left_stick_y - new_state.left_stick_y).abs() > threshold {
            changes.push(format!("Left Stick Y: {:.2}", new_state.left_stick_y));
        }
        if (old_state.right_stick_x - new_state.right_stick_x).abs() > threshold {
            changes.push(format!("Right Stick X: {:.2}", new_state.right_stick_x));
        }
        if (old_state.right_stick_y - new_state.right_stick_y).abs() > threshold {
            changes.push(format!("Right Stick Y: {:.2}", new_state.right_stick_y));
        }
        if (old_state.left_trigger - new_state.left_trigger).abs() > threshold {
            changes.push(format!("Left Trigger: {:.2}", new_state.left_trigger));
        }
        if (old_state.right_trigger - new_state.right_trigger).abs() > threshold {
            changes.push(format!("Right Trigger: {:.2}", new_state.right_trigger));
        }

        // Add changes to the log
        if !changes.is_empty() {
            if let Ok(mut inputs) = received_inputs.lock() {
                for change in changes {
                    inputs.push(format!("[{}] {}", 
                        chrono::Utc::now().format("%H:%M:%S%.3f"),
                        change
                    ));
                    info!("Input: {}", change);
                }
                
                // Keep only last 100 entries
                if inputs.len() > 100 {
                    let len = inputs.len();
                    inputs.drain(0..len - 100);
                }
            }
        }
    }

    async fn update_virtual_controller(&self, _state: &ControllerState) {
        // In a real implementation, you'd send this to ViGEm
        // For now, we'll just log that we're updating the virtual controller
        // 
        // Example with vigem-client:
        // self.vigem_client.update(XInputState {
        //     thumb_lx: (state.left_stick_x * 32767.0) as i16,
        //     thumb_ly: (state.left_stick_y * 32767.0) as i16,
        //     thumb_rx: (state.right_stick_x * 32767.0) as i16,
        //     thumb_ry: (state.right_stick_y * 32767.0) as i16,
        //     left_trigger: (state.left_trigger * 255.0) as u8,
        //     right_trigger: (state.right_trigger * 255.0) as u8,
        //     wButtons: self.build_button_mask(state),
        // });
    }

    #[allow(dead_code)]
    fn build_button_mask(&self, state: &ControllerState) -> u16 {
        let mut mask = 0u16;
        
        if state.dpad_up { mask |= 0x0001; }
        if state.dpad_down { mask |= 0x0002; }
        if state.dpad_left { mask |= 0x0004; }
        if state.dpad_right { mask |= 0x0008; }
        if state.button_start { mask |= 0x0010; }
        if state.button_back { mask |= 0x0020; }
        if state.button_l3 { mask |= 0x0040; }
        if state.button_r3 { mask |= 0x0080; }
        if state.button_lb { mask |= 0x0100; }
        if state.button_rb { mask |= 0x0200; }
        if state.button_guide { mask |= 0x0400; }
        if state.button_a { mask |= 0x1000; }
        if state.button_b { mask |= 0x2000; }
        if state.button_x { mask |= 0x4000; }
        if state.button_y { mask |= 0x8000; }
        
        mask
    }
}
