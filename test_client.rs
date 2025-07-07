use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::time::{SystemTime, UNIX_EPOCH};

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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let url = "ws://192.168.1.185:8080";
    println!("Connecting to {}", url);
    
    let (ws_stream, response) = connect_async(url).await?;
    println!("Connected! Response: {:?}", response);
    
    let (mut tx, _rx) = ws_stream.split();
    
    // Send test controller data
    for i in 0..10 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let test_data = ControllerInputData {
            timestamp,
            controller_id: 1,
            button_events: vec![ButtonEvent {
                button: "A (South)".to_string(),
                pressed: i % 2 == 0,
                timestamp,
            }],
            axis_events: vec![AxisEvent {
                axis: "Left Stick X".to_string(),
                value: (i as f32 - 5.0) / 5.0,
                timestamp,
            }],
        };
        
        let json = serde_json::to_string(&test_data)?;
        tx.send(Message::Text(json)).await?;
        
        println!("Sent test data {}", i + 1);
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
    
    println!("Test completed!");
    Ok(())
}
