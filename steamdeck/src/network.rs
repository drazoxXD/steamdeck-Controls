use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use gilrs::{GamepadId, Button, Axis};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerInputData {
    pub timestamp: u64,
    pub controller_id: u32,
    pub button_events: Vec<ButtonEvent>,
    pub axis_events: Vec<AxisEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonEvent {
    pub button: String,
    pub pressed: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisEvent {
    pub axis: String,
    pub value: f32,
    pub timestamp: u64,
}

pub struct NetworkStreamer {
    server_address: String,
    connected: bool,
    websocket: Option<Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
}

impl NetworkStreamer {
    pub fn new() -> Self {
        Self {
            server_address: String::new(),
            connected: false,
            websocket: None,
        }
    }

    pub async fn connect(&mut self, server_ip: &str, port: i32) -> Result<()> {
        self.server_address = format!("{}:{}", server_ip, port);
        let url = format!("ws://{}/controller", self.server_address);
        
        log::info!("Attempting to connect to {}", url);
        
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                self.websocket = Some(Arc::new(Mutex::new(ws_stream)));
                self.connected = true;
                log::info!("Successfully connected to server");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to connect to server: {}", e);
                self.connected = false;
                Err(anyhow::anyhow!("Failed to connect: {}", e))
            }
        }
    }

    pub fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        self.websocket = None;
        log::info!("Disconnected from server");
        Ok(())
    }

    pub fn send_controller_data(&mut self, data: ControllerInputData) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        if let Some(ref websocket) = self.websocket {
            let ws = websocket.clone();
            let json_data = serde_json::to_string(&data)?;
            
            // Use tokio::task::block_in_place to run async code in sync context
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().spawn(async move {
                    if let Ok(mut ws_lock) = ws.try_lock() {
                        if let Err(e) = ws_lock.send(Message::Text(json_data)).await {
                            log::error!("Failed to send WebSocket message: {}", e);
                        }
                    }
                });
            });
        }

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

pub fn button_to_string(button: Button) -> String {
    match button {
        Button::South => "A (South)".to_string(),
        Button::East => "B (East)".to_string(),
        Button::North => "Y (North)".to_string(),
        Button::West => "X (West)".to_string(),
        Button::LeftTrigger => "LB".to_string(),
        Button::RightTrigger => "RB".to_string(),
        Button::LeftTrigger2 => "LT".to_string(),
        Button::RightTrigger2 => "RT".to_string(),
        Button::Select => "Select".to_string(),
        Button::Start => "Start".to_string(),
        Button::Mode => "Guide".to_string(),
        Button::LeftThumb => "LSB".to_string(),
        Button::RightThumb => "RSB".to_string(),
        Button::DPadUp => "D-Pad Up".to_string(),
        Button::DPadDown => "D-Pad Down".to_string(),
        Button::DPadLeft => "D-Pad Left".to_string(),
        Button::DPadRight => "D-Pad Right".to_string(),
        _ => format!("{:?}", button),
    }
}

pub fn axis_to_string(axis: Axis) -> String {
    match axis {
        Axis::LeftStickX => "Left Stick X".to_string(),
        Axis::LeftStickY => "Left Stick Y".to_string(),
        Axis::LeftZ => "LT Axis".to_string(),
        Axis::RightStickX => "Right Stick X".to_string(),
        Axis::RightStickY => "Right Stick Y".to_string(),
        Axis::RightZ => "RT Axis".to_string(),
        Axis::DPadX => "D-Pad X".to_string(),
        Axis::DPadY => "D-Pad Y".to_string(),
        _ => format!("{:?}", axis),
    }
}

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
