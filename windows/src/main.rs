mod protocol;
mod virtual_controller;
mod ui;
mod network;

use anyhow::Result;
use eframe::egui;
use log::info;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use protocol::*;
use virtual_controller::VirtualControllerManager;
use ui::WindowsUI;
use network::NetworkClient;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Windows Controller Host");

    let (tx, rx) = mpsc::channel(100);
    let controller_state = Arc::new(Mutex::new(ControllerState::default()));
    let controller_list = Arc::new(Mutex::new(Vec::new()));
    let received_inputs = Arc::new(Mutex::new(Vec::new()));

    // Start virtual controller manager
    let virtual_controller = VirtualControllerManager::new()?;
    let controller_state_clone = controller_state.clone();
    let received_inputs_clone = received_inputs.clone();
    tokio::spawn(async move {
        virtual_controller.run(rx, controller_state_clone, received_inputs_clone).await;
    });

    // Start network client
    let network_client = NetworkClient::new();
    let tx_clone = tx.clone();
    let controller_list_clone = controller_list.clone();
    tokio::spawn(async move {
        network_client.run(tx_clone, controller_list_clone).await;
    });

    // Start UI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Windows Controller Host - SteamDeck Receiver"),
        ..Default::default()
    };

    eframe::run_native(
        "Windows Controller Host - SteamDeck Receiver",
        options,
        Box::new(|_cc| Box::new(WindowsUI::new(controller_state, controller_list, received_inputs))),
    ).map_err(|e| anyhow::anyhow!("Failed to run UI: {}", e))?;

    Ok(())
}
