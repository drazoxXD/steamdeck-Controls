use anyhow::Result;
use gilrs::{Gilrs, Event};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{
    event::{Event as WinitEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod controller_debug;
mod steam_input;
mod network;

use controller_debug::ControllerDebugUI;
use steam_input::SteamInputManager;
use network::{NetworkStreamer, ControllerInputData, ButtonEvent, AxisEvent, button_to_string, axis_to_string, get_current_timestamp};

pub struct App {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    imgui: imgui::Context,
    platform: WinitPlatform,
    renderer: Renderer,
    controller_debug: ControllerDebugUI,
    steam_input: SteamInputManager,
    gilrs: Gilrs,
    last_cursor: Option<imgui::MouseCursor>,
    network_streamer: NetworkStreamer,
    pending_connect: Option<(String, i32)>,
    pending_disconnect: bool,
    last_mirror_time: std::time::Instant,
}

impl App {
    async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(window) }?;
        
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.ok_or_else(|| anyhow::anyhow!("Failed to find suitable adapter"))?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None,
        ).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let mut imgui = imgui::Context::create();
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);

        let renderer_config = RendererConfig {
            texture_format: surface_format,
            ..Default::default()
        };
        
        let renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

        let controller_debug = ControllerDebugUI::new();
        let steam_input = SteamInputManager::new()?;
        let gilrs = Gilrs::new().unwrap();

        let network_streamer = NetworkStreamer::new();

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            imgui,
            platform,
            renderer,
            controller_debug,
            steam_input,
            gilrs,
            last_cursor: None,
            network_streamer,
            pending_connect: None,
            pending_disconnect: false,
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width.max(1);
            self.config.height = new_size.height.max(1);
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent, window: &Window) -> bool {
        // Create a WindowEvent that owns the data
        let owned_event = match event {
            WindowEvent::CloseRequested => WindowEvent::CloseRequested,
            WindowEvent::Resized(size) => WindowEvent::Resized(*size),

            WindowEvent::CursorMoved { device_id, position, modifiers } => {
                WindowEvent::CursorMoved { device_id: *device_id, position: *position, modifiers: *modifiers }
            }
            WindowEvent::CursorEntered { device_id } => {
                WindowEvent::CursorEntered { device_id: *device_id }
            }
            WindowEvent::CursorLeft { device_id } => {
                WindowEvent::CursorLeft { device_id: *device_id }
            }
            WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => {
                WindowEvent::MouseWheel { device_id: *device_id, delta: *delta, phase: *phase, modifiers: *modifiers }
            }
            WindowEvent::MouseInput { device_id, state, button, modifiers } => {
                WindowEvent::MouseInput { device_id: *device_id, state: *state, button: *button, modifiers: *modifiers }
            }
            WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                WindowEvent::KeyboardInput { device_id: *device_id, input: *input, is_synthetic: *is_synthetic }
            }
            _ => return false, // Skip other events
        };
        
        let winit_event = WinitEvent::WindowEvent { 
            window_id: window.id(), 
            event: owned_event 
        };
        
        self.platform.handle_event::<()>(self.imgui.io_mut(), window, &winit_event);
        false
    }

    fn update(&mut self) {
        // Handle pending network operations
        if let Some((ip, port)) = self.pending_connect.take() {
            let mut network_streamer = NetworkStreamer::new();
            
            // Use tokio::task::block_in_place to run async code in sync context
            let connection_result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(network_streamer.connect(&ip, port))
            });
            
            match connection_result {
                Ok(_) => {
                    self.network_streamer = network_streamer;
                    self.controller_debug.set_connection_status("Connected".to_string());
                    self.controller_debug.set_network_enabled(true);
                    log::info!("Successfully connected to server");
                }
                Err(e) => {
                    self.controller_debug.set_connection_status("Connection Failed".to_string());
                    self.controller_debug.set_network_enabled(false);
                    log::error!("Failed to connect to server: {}", e);
                }
            }
        }

        if self.pending_disconnect {
            self.pending_disconnect = false;
            let _ = self.network_streamer.disconnect();
            self.controller_debug.set_connection_status("Disconnected".to_string());
            self.controller_debug.set_network_enabled(false);
        }

        // Check for UI-triggered network operations
        if let Some((server_ip, server_port)) = self.controller_debug.should_connect_network() {
            if !self.network_streamer.is_connected() && self.pending_connect.is_none() {
                self.pending_connect = Some((server_ip, server_port));
            }
        }
        
        if self.controller_debug.should_disconnect_network() {
            self.pending_disconnect = true;
        }
        
        // Poll controller events
        let mut network_data = ControllerInputData {
            timestamp: get_current_timestamp(),
            controller_id: 0,
            button_events: Vec::new(),
            axis_events: Vec::new(),
        };

        while let Some(Event { id, event, time }) = self.gilrs.next_event() {
            // Update controller debug UI
            self.controller_debug.handle_gilrs_event(id, event, time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64());
            
            // Prepare network data
            network_data.controller_id = usize::from(id) as u32;
            let timestamp = get_current_timestamp();
            
            // Update Steam Input with real controller data
            match event {
                gilrs::EventType::Connected => {
                    log::info!("Controller {} connected", id);
                    
                    // Auto-connect to server when controller connects
                    if !self.network_streamer.is_connected() {
                        log::info!("Auto-connecting to server...");
                        self.controller_debug.set_connection_status("Connecting...".to_string());
                        
                        // We'll handle this in the render loop since we can't do async here
                    }
                }
                gilrs::EventType::Disconnected => {
                    log::info!("Controller {} disconnected", id);
                    self.steam_input.remove_controller(id);
                }
                gilrs::EventType::ButtonPressed(button, _) => {
                    self.steam_input.update_from_controller_input(id, Some((button, true)), None);
                    
                    // Add to network data
                    network_data.button_events.push(ButtonEvent {
                        button: button_to_string(button),
                        pressed: true,
                        timestamp,
                    });
                    
                    log::info!("Button pressed: {:?}", button);
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    self.steam_input.update_from_controller_input(id, Some((button, false)), None);
                    
                    // Add to network data
                    network_data.button_events.push(ButtonEvent {
                        button: button_to_string(button),
                        pressed: false,
                        timestamp,
                    });
                    
                    log::info!("Button released: {:?}", button);
                }
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    self.steam_input.update_from_controller_input(id, None, Some((axis, value)));
                    
                    // Only send significant axis changes to reduce network traffic
                    if value.abs() > 0.1 {
                        network_data.axis_events.push(AxisEvent {
                            axis: axis_to_string(axis),
                            value,
                            timestamp,
                        });
                    }
                }
                gilrs::EventType::ButtonChanged(button, value, _) => {
                    // Treat as digital input with threshold
                    let pressed = value > 0.5;
                    self.steam_input.update_from_controller_input(id, Some((button, pressed)), None);
                    
                    // Add to network data
                    network_data.button_events.push(ButtonEvent {
                        button: button_to_string(button),
                        pressed,
                        timestamp,
                    });
                }
                _ => {}
            }
        }

        // Send network data if we have events and are connected
        if (!network_data.button_events.is_empty() || !network_data.axis_events.is_empty()) && self.network_streamer.is_connected() {
            log::info!("Sending {} button events and {} axis events", 
                network_data.button_events.len(), 
                network_data.axis_events.len());
                
            // Try to send the data
            if let Err(e) = self.network_streamer.send_controller_data(network_data) {
                log::error!("Failed to send network data: {}", e);
            }
        }

        // Update Steam Input (this now just maintains internal state)
        self.steam_input.update();
        
        // Update controller debug UI with Steam Input data
        self.controller_debug.update_steam_input(&self.steam_input);
    }

    fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        self.platform.prepare_frame(self.imgui.io_mut(), window).expect("Failed to prepare frame");
        let ui = self.imgui.frame();

        // Render controller debug UI
        self.controller_debug.render(&ui);

        // Handle cursor before rendering
        let cursor = ui.mouse_cursor();
        if self.last_cursor != cursor {
            self.last_cursor = cursor;
            self.platform.prepare_render(&ui, window);
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        let draw_data = self.imgui.render();
        self.renderer.render(&draw_data, &self.queue, &self.device, &mut render_pass)
            .expect("Rendering failed");

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

async fn run() -> Result<()> {
    env_logger::init();
    
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Steam Deck Controller Debug UI")
        .with_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)?;

    let mut app = App::new(&window).await?;

    event_loop.run(move |event, _, control_flow| {
        match event {
            WinitEvent::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !app.input(event, &window) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            app.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            app.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            WinitEvent::RedrawRequested(window_id) if window_id == window.id() => {
                app.update();
                match app.render(&window) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => app.resize(app.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            WinitEvent::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

fn main() -> Result<()> {
    // Use Tokio runtime instead of pollster
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(run())
}
