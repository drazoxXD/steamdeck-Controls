mod protocol;
mod controller;
#[cfg(feature = "gui")]
mod ui;
mod network;

use anyhow::Result;
use log::info;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use protocol::*;
use controller::ControllerManager;
use network::NetworkManager;

#[cfg(feature = "gui")]
use ui::SteamDeckUI;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting SteamDeck Controller Client");

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

    #[cfg(feature = "gui")]
    {
        // Start UI
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800.0, 600.0])
                .with_title("SteamDeck Controller Client"),
            ..Default::default()
        };

        eframe::run_native(
            "SteamDeck Controller Client",
            options,
            Box::new(|_cc| Box::new(SteamDeckUI::new(controller_state, controller_list))),
        ).map_err(|e| anyhow::anyhow!("Failed to run UI: {}", e))?;
    }

    #[cfg(not(feature = "gui"))]
    {
        info!("Running in console mode - GUI disabled");
        info!("Press Ctrl+C to exit");
        
        // Keep the main thread alive
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    Ok(())
}
