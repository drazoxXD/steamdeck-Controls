use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerState {
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub button_a: bool,
    pub button_b: bool,
    pub button_x: bool,
    pub button_y: bool,
    pub button_lb: bool,
    pub button_rb: bool,
    pub button_back: bool,
    pub button_start: bool,
    pub button_guide: bool,
    pub button_l3: bool,
    pub button_r3: bool,
    pub timestamp: u64,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            left_stick_x: 0.0,
            left_stick_y: 0.0,
            right_stick_x: 0.0,
            right_stick_y: 0.0,
            left_trigger: 0.0,
            right_trigger: 0.0,
            dpad_up: false,
            dpad_down: false,
            dpad_left: false,
            dpad_right: false,
            button_a: false,
            button_b: false,
            button_x: false,
            button_y: false,
            button_lb: false,
            button_rb: false,
            button_back: false,
            button_start: false,
            button_guide: false,
            button_l3: false,
            button_r3: false,
            timestamp: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerInfo {
    pub name: String,
    pub uuid: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    ControllerList(Vec<ControllerInfo>),
    ControllerState(ControllerState),
    Ping,
    Pong,
}

pub const PROTOCOL_VERSION: u8 = 1;
pub const USB_VENDOR_ID: u16 = 0x1234;
pub const USB_PRODUCT_ID: u16 = 0x5678;
pub const NETWORK_PORT: u16 = 12345;
