use anyhow::Result;
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::{
    event::{Event as WinitEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

mod controller_receiver;
use controller_receiver::ControllerReceiver;

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

pub struct App {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    imgui: imgui::Context,
    platform: WinitPlatform,
    renderer: Renderer,
    controller_receiver: ControllerReceiver,
    last_cursor: Option<imgui::MouseCursor>,
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
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
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

        let controller_receiver = ControllerReceiver::new();

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            imgui,
            platform,
            renderer,
            controller_receiver,
            last_cursor: None,
        })
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
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
            _ => return false,
        };
        
        let winit_event = WinitEvent::WindowEvent { 
            window_id: window.id(), 
            event: owned_event 
        };
        
        self.platform.handle_event::<()>(self.imgui.io_mut(), window, &winit_event);
        false
    }

    fn update(&mut self) {
        self.controller_receiver.update();
    }

    fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        self.platform.prepare_frame(self.imgui.io_mut(), window).expect("Failed to prepare frame");
        let ui = self.imgui.frame();

        // Render controller receiver UI
        self.controller_receiver.render(&ui);

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
        .with_title("Steam Deck Controller Server")
        .with_inner_size(winit::dpi::LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)?;

    let mut app = App::new(&window).await?;

    // Start the WebSocket server
    let server_handle = tokio::spawn(async {
        start_websocket_server().await
    });

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

async fn start_websocket_server() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    log::info!("WebSocket server listening on 0.0.0.0:8080");

    while let Ok((stream, addr)) = listener.accept().await {
        log::info!("New connection from {}", addr);
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                log::error!("Error handling connection: {}", e);
            }
        });
    }
    
    Ok(())
}

async fn handle_connection(stream: tokio::net::TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await?;
    let (mut tx, mut rx) = ws_stream.split();
    
    log::info!("WebSocket connection established");
    
    while let Some(msg) = rx.next().await {
        match msg? {
            Message::Text(text) => {
                if let Ok(controller_data) = serde_json::from_str::<ControllerInputData>(&text) {
                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    
                    let delay = if controller_data.timestamp < current_time {
                        current_time - controller_data.timestamp
                    } else {
                        0
                    };
                    
                    for button_event in &controller_data.button_events {
                        println!("Button: {} - {} ({}ms delay)", 
                            button_event.button, 
                            if button_event.pressed { "Pressed" } else { "Released" },
                            delay);
                    }
                    
                    for axis_event in &controller_data.axis_events {
                        println!("Axis: {} - {:.3} ({}ms delay)", 
                            axis_event.axis, 
                            axis_event.value,
                            delay);
                    }
                }
            }
            Message::Close(_) => {
                log::info!("WebSocket connection closed");
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run())
}
