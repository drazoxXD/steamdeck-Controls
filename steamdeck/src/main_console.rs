mod protocol;
mod controller;
mod network;

use anyhow::Result;
use log::info;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use protocol::*;
use controller::ControllerManager;
use network::NetworkManager;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting SteamDeck Controller Client (Console Mode)");

    let (tx, rx) = mpsc::channel(100);
    let controller_state = Arc::new(Mutex::new(ControllerState::default()));
    let controller_list = Arc::new(Mutex::new(Vec::new()));

    // Start controller manager
    let mut controller_manager = ControllerManager::new(tx.clone());
    let controller_list_clone = controller_list.clone();
    tokio::spawn(async move {
        controller_manager.run(controller_list_clone).await;
    });

    // Start network manager
    let mut network_manager = NetworkManager::new();
    let controller_state_clone = controller_state.clone();
    let controller_list_clone = controller_list.clone();
    tokio::spawn(async move {
        network_manager.run(rx, controller_state_clone, controller_list_clone).await;
    });

    // Console output loop
    let mut last_timestamp = 0u64;
    
    info!("=== SteamDeck Controller Client Started ===");
    info!("Listening for Windows host connections on port {}", NETWORK_PORT);
    info!("Press Ctrl+C to exit");

    loop {
        // Print controller list periodically
        if let Ok(controllers) = controller_list.lock() {
            if !controllers.is_empty() {
                println!("\n=== Available Controllers ===");
                for controller in controllers.iter() {
                    let status = if controller.connected { "🟢 CONNECTED" } else { "🔴 DISCONNECTED" };
                    println!("  {} - {} (VID: {:04X}, PID: {:04X})", 
                        status, controller.name, controller.vendor_id, controller.product_id);
                }
            }
        }

        // Print controller state changes
        if let Ok(state) = controller_state.lock() {
            if state.timestamp != last_timestamp && state.timestamp > 0 {
                println!("\n=== Controller Input ===");
                
                // Print non-zero analog values
                if state.left_stick_x.abs() > 0.1 || state.left_stick_y.abs() > 0.1 {
                    println!("  Left Stick: X={:.2}, Y={:.2}", state.left_stick_x, state.left_stick_y);
                }
                if state.right_stick_x.abs() > 0.1 || state.right_stick_y.abs() > 0.1 {
                    println!("  Right Stick: X={:.2}, Y={:.2}", state.right_stick_x, state.right_stick_y);
                }
                if state.left_trigger > 0.1 {
                    println!("  Left Trigger: {:.2}", state.left_trigger);
                }
                if state.right_trigger > 0.1 {
                    println!("  Right Trigger: {:.2}", state.right_trigger);
                }

                // Print pressed buttons
                let mut pressed_buttons = Vec::new();
                if state.button_a { pressed_buttons.push("A"); }
                if state.button_b { pressed_buttons.push("B"); }
                if state.button_x { pressed_buttons.push("X"); }
                if state.button_y { pressed_buttons.push("Y"); }
                if state.button_lb { pressed_buttons.push("LB"); }
                if state.button_rb { pressed_buttons.push("RB"); }
                if state.button_l3 { pressed_buttons.push("L3"); }
                if state.button_r3 { pressed_buttons.push("R3"); }
                if state.button_start { pressed_buttons.push("START"); }
                if state.button_back { pressed_buttons.push("BACK"); }
                if state.button_guide { pressed_buttons.push("GUIDE"); }
                if state.dpad_up { pressed_buttons.push("D-UP"); }
                if state.dpad_down { pressed_buttons.push("D-DOWN"); }
                if state.dpad_left { pressed_buttons.push("D-LEFT"); }
                if state.dpad_right { pressed_buttons.push("D-RIGHT"); }

                if !pressed_buttons.is_empty() {
                    println!("  Pressed Buttons: {}", pressed_buttons.join(", "));
                }

                last_timestamp = state.timestamp;
            }
        }

        sleep(Duration::from_millis(100)).await;
    }
}
