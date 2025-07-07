use anyhow::Result;
use vigem_client::{Client, Xbox360Wired};
use std::collections::HashMap;
use crate::ControllerInputData;

pub struct VirtualController {
    client: Client,
    target: Option<Xbox360Wired<Client>>,
    gamepad_state: vigem_client::XGamepad,
    button_states: HashMap<String, bool>,
    axis_states: HashMap<String, f32>,
}

impl VirtualController {
    pub fn new() -> Result<Self> {
        let client = Client::connect()?;
        
        Ok(Self {
            client,
            target: None,
            gamepad_state: vigem_client::XGamepad::default(),
            button_states: HashMap::new(),
            axis_states: HashMap::new(),
        })
    }

    pub fn create_controller(&mut self) -> Result<()> {
        // Create a new target and get its ID
        let mut target = Xbox360Wired::new(self.client.try_clone()?, vigem_client::TargetId::XBOX360_WIRED);
        
        // Connect the target
        target.plugin()?;
        
        self.target = Some(target);
        
        log::info!("Virtual Xbox 360 controller created successfully");
        Ok(())
    }

    pub fn disconnect_controller(&mut self) -> Result<()> {
        if let Some(mut target) = self.target.take() {
            target.unplug()?;
            log::info!("Virtual Xbox 360 controller disconnected");
        }
        Ok(())
    }

    pub fn process_controller_input(&mut self, input: ControllerInputData) -> Result<()> {
        if self.target.is_none() {
            return Ok(());
        }

        // Process button events
        for button_event in input.button_events {
            self.button_states.insert(button_event.button.clone(), button_event.pressed);
            self.update_button_state(&button_event.button, button_event.pressed);
        }

        // Process axis events
        for axis_event in input.axis_events {
            self.axis_states.insert(axis_event.axis.clone(), axis_event.value);
            self.update_axis_state(&axis_event.axis, axis_event.value);
        }

        // Update the virtual controller
        self.update_virtual_controller()?;

        Ok(())
    }

    fn update_button_state(&mut self, button: &str, pressed: bool) {
        use vigem_client::XButtons;

        let button_flag = match button {
            "A (South)" => XButtons::A,
            "B (East)" => XButtons::B,
            "X (West)" => XButtons::X,
            "Y (North)" => XButtons::Y,
            "LB" => XButtons::LB,
            "RB" => XButtons::RB,
            "Select" => XButtons::BACK,
            "Start" => XButtons::START,
            "Guide" => XButtons::GUIDE,
            "LSB" => XButtons::LTHUMB,
            "RSB" => XButtons::RTHUMB,
            "D-Pad Up" => XButtons::UP,
            "D-Pad Down" => XButtons::DOWN,
            "D-Pad Left" => XButtons::LEFT,
            "D-Pad Right" => XButtons::RIGHT,
            // Handle RT/LT as digital buttons too
            "RT [ID: 7] - Fire" | "LT [ID: 6] - Aim" => {
                // For RT/LT, set the trigger to 100% when pressed, 0% when released

                if button.contains("RT") {
                    self.gamepad_state.right_trigger = if pressed { 255 } else { 0 };
                    log::info!("RT digital button: {} -> trigger value: {}", pressed, self.gamepad_state.right_trigger);
                } else if button.contains("LT") {
                    self.gamepad_state.left_trigger = if pressed { 255 } else { 0 };
                    log::info!("LT digital button: {} -> trigger value: {}", pressed, self.gamepad_state.left_trigger);
                }
                return; // Don't process as normal button
            }
            _ => return,
        };
        if pressed {
            self.gamepad_state.buttons.raw |= button_flag;
        } else {
            self.gamepad_state.buttons.raw &= !button_flag;
        }
    }

    fn update_axis_state(&mut self, axis: &str, value: f32) {
        match axis {
            "Left Stick X" => {
                self.gamepad_state.thumb_lx = (value * 32767.0) as i16;
            }
            "Left Stick Y" => {
                // Don't invert Y axis - use raw value
                self.gamepad_state.thumb_ly = (value * 32767.0) as i16;
            }
            "Right Stick X" => {
                self.gamepad_state.thumb_rx = (value * 32767.0) as i16;
            }
            "Right Stick Y" => {
                // Don't invert Y axis - use raw value
                self.gamepad_state.thumb_ry = (value * 32767.0) as i16;
            }
            "LT Axis" => {
                self.gamepad_state.left_trigger = (value * 255.0) as u8;
            }
            "RT Axis" => {
                self.gamepad_state.right_trigger = (value * 255.0) as u8;
            }
            _ => {}
        }
    }

    fn update_virtual_controller(&mut self) -> Result<()> {
        if let Some(target) = &mut self.target {
            target.update(&self.gamepad_state)?;
        }
        Ok(())
    }

    pub fn get_button_states(&self) -> &HashMap<String, bool> {
        &self.button_states
    }

    pub fn get_axis_states(&self) -> &HashMap<String, f32> {
        &self.axis_states
    }

    pub fn is_connected(&self) -> bool {
        self.target.is_some()
    }
}

impl std::fmt::Debug for VirtualController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualController")
            .field("is_connected", &self.is_connected())
            .field("button_states", &self.button_states)
            .field("axis_states", &self.axis_states)
            .finish()
    }
}

impl Drop for VirtualController {
    fn drop(&mut self) {
        let _ = self.disconnect_controller();
    }
}
