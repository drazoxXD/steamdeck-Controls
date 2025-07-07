use anyhow::Result;
use log::{info, warn, error};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::protocol::*;

pub struct NetworkClient {
    stream: Option<TcpStream>,
}

impl NetworkClient {
    pub fn new() -> Self {
        Self {
            stream: None,
        }
    }

    pub async fn run(
        &mut self,
        controller_tx: mpsc::Sender<ControllerState>,
        controller_list: Arc<Mutex<Vec<ControllerInfo>>>,
    ) {
        info!("Starting network client");

        loop {
            // Try to connect to SteamDeck
            match self.connect_to_steamdeck().await {
                Ok(()) => {
                    info!("Connected to SteamDeck");
                    
                    // Handle the connection
                    if let Err(e) = self.handle_connection(&controller_tx, &controller_list).await {
                        error!("Connection error: {}", e);
                    }
                    
                    info!("Disconnected from SteamDeck");
                }
                Err(e) => {
                    warn!("Failed to connect to SteamDeck: {}", e);
                }
            }

            // Wait before retrying
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_to_steamdeck(&mut self) -> Result<()> {
        // Try to connect to localhost first (for testing)
        match TcpStream::connect(format!("127.0.0.1:{}", NETWORK_PORT)).await {
            Ok(stream) => {
                self.stream = Some(stream);
                return Ok(());
            }
            Err(_) => {
                // Try to find SteamDeck on local network
                // You could implement mDNS discovery here
                // For now, try common IP ranges
                for i in 1..255 {
                    let ip = format!("192.168.1.{}:{}", i, NETWORK_PORT);
                    match TcpStream::connect(&ip).await {
                        Ok(stream) => {
                            info!("Found SteamDeck at {}", ip);
                            self.stream = Some(stream);
                            return Ok(());
                        }
                        Err(_) => continue,
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Could not find SteamDeck"))
    }

    async fn handle_connection(
        &mut self,
        controller_tx: &mpsc::Sender<ControllerState>,
        controller_list: &Arc<Mutex<Vec<ControllerInfo>>>,
    ) -> Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| anyhow::anyhow!("No stream"))?;
        
        loop {
            // Read message length
            let mut len_bytes = [0u8; 4];
            match stream.read_exact(&mut len_bytes).await {
                Ok(_) => {
                    let len = u32::from_le_bytes(len_bytes) as usize;
                    
                    // Read message
                    let mut buffer = vec![0u8; len];
                    stream.read_exact(&mut buffer).await?;
                    
                    // Parse message
                    let json = String::from_utf8(buffer)?;
                    let message: Message = serde_json::from_str(&json)?;
                    
                    // Handle message
                    match message {
                        Message::ControllerList(controllers) => {
                            info!("Received controller list: {} controllers", controllers.len());
                            if let Ok(mut list) = controller_list.lock() {
                                *list = controllers;
                            }
                        }
                        Message::ControllerState(state) => {
                            if let Err(e) = controller_tx.send(state).await {
                                error!("Failed to send controller state: {}", e);
                            }
                        }
                        Message::Ping => {
                            // Respond with pong
                            let pong = Message::Pong;
                            self.send_message(pong).await?;
                        }
                        Message::Pong => {
                            // Handle pong if needed
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from stream: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn send_message(&mut self, message: Message) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            let json = serde_json::to_string(&message)?;
            let len = json.len() as u32;
            
            stream.write_all(&len.to_le_bytes()).await?;
            stream.write_all(json.as_bytes()).await?;
            stream.flush().await?;
        }
        
        Ok(())
    }
}
