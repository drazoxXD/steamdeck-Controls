use anyhow::Result;
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::protocol::*;

pub struct NetworkManager {
    listener: Option<TcpListener>,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            listener: None,
        }
    }

    pub async fn run(
        &mut self,
        mut controller_rx: mpsc::Receiver<ControllerState>,
        controller_state: Arc<Mutex<ControllerState>>,
        controller_list: Arc<Mutex<Vec<ControllerInfo>>>,
    ) {
        info!("Starting network manager");

        // Try to bind to the network port
        match TcpListener::bind(format!("0.0.0.0:{}", NETWORK_PORT)).await {
            Ok(listener) => {
                info!("Network server listening on port {}", NETWORK_PORT);
                self.listener = Some(listener);
            }
            Err(e) => {
                error!("Failed to bind to network port: {}", e);
                return;
            }
        }

        let listener = self.listener.as_ref().unwrap();

        loop {
            tokio::select! {
                // Accept new connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            info!("New connection from {}", addr);
                            let controller_state_clone = controller_state.clone();
                            let controller_list_clone = controller_list.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_client(stream, controller_state_clone, controller_list_clone).await {
                                    error!("Client handler error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }

                // Update controller state from controller manager
                state = controller_rx.recv() => {
                    if let Some(state) = state {
                        if let Ok(mut current_state) = controller_state.lock() {
                            *current_state = state;
                        }
                    }
                }
            }
        }
    }
}

async fn handle_client(
    mut stream: TcpStream,
    controller_state: Arc<Mutex<ControllerState>>,
    controller_list: Arc<Mutex<Vec<ControllerInfo>>>,
) -> Result<()> {
    let mut buffer = [0; 1024];
    let mut last_state = ControllerState::default();

    // Send initial controller list
    if let Ok(controllers) = controller_list.lock() {
        let message = Message::ControllerList(controllers.clone());
        send_message(&mut stream, &message).await?;
    }

    loop {
        tokio::select! {
            // Read from client
            result = stream.read(&mut buffer) => {
                match result {
                    Ok(0) => {
                        info!("Client disconnected");
                        break;
                    }
                    Ok(_n) => {
                        // Handle incoming messages (if any)
                        // For now, we mainly send data to the client
                    }
                    Err(e) => {
                        error!("Error reading from client: {}", e);
                        break;
                    }
                }
            }

            // Send controller state updates
            _ = sleep(Duration::from_millis(16)) => {
                if let Ok(state) = controller_state.lock() {
                    // Only send if state changed
                    if state.timestamp != last_state.timestamp {
                        let message = Message::ControllerState(state.clone());
                        if let Err(e) = send_message(&mut stream, &message).await {
                            error!("Failed to send controller state: {}", e);
                            break;
                        }
                        last_state = state.clone();
                    }
                }
            }
        }
    }

    Ok(())
}

async fn send_message(stream: &mut TcpStream, message: &Message) -> Result<()> {
    let json = serde_json::to_string(message)?;
    let len = json.len() as u32;
    
    // Send length first, then message
    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(json.as_bytes()).await?;
    stream.flush().await?;
    
    Ok(())
}
