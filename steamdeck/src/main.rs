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

use controller_debug::ControllerDebugUI;
use steam_input::SteamInputManager;

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

        let controller_debug = ControllerDebugUI::new();
        let steam_input = SteamInputManager::new()?;
        let gilrs = Gilrs::new().unwrap();

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
        // Poll controller events
        while let Some(Event { id, event, time }) = self.gilrs.next_event() {
            // Update controller debug UI
            self.controller_debug.handle_gilrs_event(id, event, time.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64());
            
            // Update Steam Input with real controller data
            match event {
                gilrs::EventType::Connected => {
                    log::info!("Controller {} connected", id);
                }
                gilrs::EventType::Disconnected => {
                    log::info!("Controller {} disconnected", id);
                    self.steam_input.remove_controller(id);
                }
                gilrs::EventType::ButtonPressed(button, _) => {
                    self.steam_input.update_from_controller_input(id, Some((button, true)), None);
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    self.steam_input.update_from_controller_input(id, Some((button, false)), None);
                }
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    self.steam_input.update_from_controller_input(id, None, Some((axis, value)));
                }
                gilrs::EventType::ButtonChanged(button, value, _) => {
                    // Treat as digital input with threshold
                    let pressed = value > 0.5;
                    self.steam_input.update_from_controller_input(id, Some((button, pressed)), None);
                }
                _ => {}
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
    pollster::block_on(run())
}
