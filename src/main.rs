use std::num::NonZeroU32;

use log::Level;
use nes::*;
use winit::{
    dpi::{LogicalSize, Size},
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
};

mod winit_app;
fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();
    let event_loop = EventLoop::new().unwrap();

    let app = winit_app::WinitAppBuilder::with_init(
        |elwt| {
            let window = winit_app::make_window(elwt, |w| {
                w.with_inner_size(Size::Logical(LogicalSize::new(320., 320.)))
            });
            let context = softbuffer::Context::new(window.clone()).unwrap();
            let mut cpu = Cpu::new();
            cpu.load_to(0x600, GAME_CODE);
            cpu.memory.write_u16(0xFFFC, 0x600);
            cpu.reset();
            let doublebuffer = [0u32; 1024];
            (window, context, cpu, doublebuffer)
        },
        |_elwt, (window, context, _cpu, _doublebuffer)| {
            softbuffer::Surface::new(context, window.clone()).unwrap()
        },
    )
    .with_event_handler(
        |(window, _context, cpu, doublebuffer), surface, event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            window.request_redraw();

            let surface = surface.unwrap();

            match event {
                Event::WindowEvent {
                    window_id: _winid,
                    event: WindowEvent::RedrawRequested,
                } => {
                    cpu.memory.write(0xfe, fastrand::u8(1..16));

                    let size = window.inner_size();
                    if let (Some(_width), Some(_height)) =
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    {
                        let mut buffer = surface.buffer_mut().unwrap();

                        if buffer.len() < 1024 * 100 {
                            return;
                        }

                        loop {
                            if cpu.status.contains(Flags::BREAK) {
                                std::process::exit(0);
                            }
                            cpu.step();

                            if read_screen_state(cpu, doublebuffer) {
                                for (i, dblbfr) in doublebuffer.iter().take(0x400).enumerate() {
                                    let y = i / 32;
                                    let x = i % 32;
                                    for dx in 0..10 {
                                        for dy in 0..10 {
                                            let coord = (y * 10 + dy) * (32 * 10) + (x * 10 + dx);

                                            buffer[coord] = *dblbfr;
                                        }
                                    }
                                }
                                // dbg!(doublebuffer);
                                buffer.present().unwrap();
                                break;
                            }
                            ::std::thread::sleep(std::time::Duration::new(0, 70_000));
                        }
                    }
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::Resized(size),
                } if window_id == window.id() => {
                    if let (Some(width), Some(height)) =
                        (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                    {
                        surface.resize(width, height).unwrap();
                    }
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key: Key::Named(NamedKey::Escape),
                                    ..
                                },
                            ..
                        },
                    window_id,
                } if window_id == window.id() => {
                    elwt.exit();
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            device_id: _devid,
                            event:
                                KeyEvent {
                                    physical_key:
                                        PhysicalKey::Code(
                                            key @ (KeyCode::KeyW
                                            | KeyCode::KeyA
                                            | KeyCode::KeyS
                                            | KeyCode::KeyD),
                                        ),
                                    ..
                                },
                            is_synthetic: _issynth,
                        },
                    ..
                } => {
                    let val = match key {
                        KeyCode::KeyW => 0x77,
                        KeyCode::KeyS => 0x73,
                        KeyCode::KeyA => 0x61,
                        KeyCode::KeyD => 0x64,
                        _ => unreachable!(),
                    };
                    cpu.memory.write(0xff, val);
                }
                _ => {}
            }
        },
    );

    winit_app::run_app(event_loop, app);
}

fn color(byte: u8) -> u32 {
    match byte {
        0 => 0x000000,
        1 => 0xFFFFFF,
        2 | 9 => 0xAAAAAA,
        3 | 10 => 0xFF0000,
        4 | 11 => 0x00FF00,
        5 | 12 => 0x0000FF,
        6 | 13 => 0xFF00FF,
        7 | 14 => 0xFFCC00,
        _ => 0x00FFCC,
    }
}

fn read_screen_state(cpu: &Cpu, frame: &mut [u32]) -> bool {
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_idx = cpu.memory.read(i as u16);
        let c = color(color_idx);
        let init = i - 0x200;
        if frame[init] != c {
            frame[init] = c;
            update = true;
        }
    }
    update
}

static GAME_CODE: &[u8] = &[
    0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9, 0x02, 0x85,
    0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9, 0x0f, 0x85,
    0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe,
    0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3,
    0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9,
    0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60,
    0xa9, 0x04, 0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
    0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02,
    0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06,
    0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07,
    0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60, 0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06,
    0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c,
    0x35, 0x07, 0x60, 0xa6, 0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02,
    0x4a, 0xb0, 0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
    0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
    0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10, 0xb0,
    0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29,
    0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35, 0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60,
    0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa6, 0xff, 0xea,
    0xea, 0xca, 0xd0, 0xfb, 0x60,
];
