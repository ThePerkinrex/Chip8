mod cpu;
mod keyboard;
mod renderer;
mod speaker;

use std::time::Instant;

use cpu::Cpu;
use keyboard::Keyboard;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use renderer::Renderer;
use speaker::Speaker;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
// use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

const TARGET_FPS: f64 = 60.;
const TARGET_INTERVAL: f64 = 1. / TARGET_FPS;

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("error,chip8"))
        .init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let (window, _, _, mut _hidpi_factor) = create_window("Chip8", &event_loop);

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut renderer = Renderer::new();
    let mut keyboard = Keyboard::new();
    let speaker = Speaker::new();
    let mut cpu = Cpu::new();
    // renderer.set_pixel(0, 0);
    // renderer.set_pixel(5, 2);
    cpu.load_rom_with_name("ROM").unwrap();
    let mut deltat = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            renderer.draw(pixels.get_frame());
            // log::info!("RENDER {:?}", t.elapsed());
            // t = Instant::now();
            // std::thread::sleep(std::time::Duration::from_millis(100));
            if let Err(e) = pixels.render() {
                match e {
                    Error::Surface(pixels::wgpu::SurfaceError::Timeout) => (), // Timeouts are ignored
                    _ => {
                        error!("pixels.render() failed: {}", e);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }
            }
        }

        if !if let Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(keycode),
                            state,
                            ..
                        },
                    ..
                },
            window_id: _,
        } = event
        {
            match state {
                ElementState::Pressed => keyboard.key_down(keycode),
                ElementState::Released => keyboard.key_up(keycode),
            }
        } else {
            false
        } && input.update(&event)
        {
            // Handle input events
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw

            // renderer.update();
            if deltat.elapsed().as_secs_f64() > TARGET_INTERVAL {
                cpu.cycle(&speaker, &mut renderer, &mut keyboard);
                deltat = Instant::now();
                window.request_redraw();
            }
        }
    });
}
fn create_window(
    title: &str,
    event_loop: &EventLoop<()>,
) -> (winit::window::Window, u32, u32, f64) {
    // Create a hidden window so we can estimate a good default window size
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(event_loop)
        .unwrap();
    let hidpi_factor = window.scale_factor();

    // Get dimensions
    let width = WIDTH as f64;
    let height = HEIGHT as f64;
    let (monitor_width, monitor_height) = {
        if let Some(monitor) = window.current_monitor() {
            let size = monitor.size().to_logical(hidpi_factor);
            (size.width, size.height)
        } else {
            (width, height)
        }
    };
    let scale = (monitor_height / (height * 2.0)).round().max(1.0);

    // Resize, center, and display the window
    let min_size: winit::dpi::LogicalSize<f64> =
        PhysicalSize::new(width, height).to_logical(hidpi_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let size = default_size.to_physical::<f64>(hidpi_factor);

    (
        window,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi_factor,
    )
}
