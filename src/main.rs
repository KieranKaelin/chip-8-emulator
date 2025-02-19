use chip_8::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH, STARTING_PC};
use rodio::source::{SineWave, Source};
use rodio::{OutputStream, Sink};
use std::fs;
use std::process::exit;
use std::time::Duration;
use std::time::Instant;
use winit::event::ElementState;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

const TIMERS_HZ: f32 = 60.0;
const INSTRUCTIONS_HZ: f32 = 700.0;

pub fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let window = WindowBuilder::new()
        .with_title("Pixels Example")
        .with_inner_size(LogicalSize::new(
            DISPLAY_WIDTH as f64 * 10.0,
            DISPLAY_HEIGHT as f64 * 10.0,
        ))
        .build(&event_loop)
        .expect("failed to create window");

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let mut chip8 = Chip8::new(&window);

    // Load the rom
    let rom: Vec<u8> = fs::read("roms/space_invaders.ch8").expect("failed to read rom");
    chip8.load_into_memory(STARTING_PC as usize, rom);

    let mut pressed_inputs = [false; KEYS.len()];

    let mut last_instant = Instant::now();

    event_loop
        .run(|event, _| match event {
            Event::AboutToWait => {
                let now = Instant::now();
                let delta = (now - last_instant).as_secs_f32();
                last_instant = now;

                let mut beep = false;
                for _ in 0..(delta * TIMERS_HZ).round() as usize {
                    beep = chip8.tick_timers_and_check_beep() || beep;
                }

                if beep {
                    sink.append(SineWave::new(440.0).take_duration(Duration::from_secs(1)));
                    sink.play();
                } else {
                    sink.clear();
                }

                for _ in 0..(delta * INSTRUCTIONS_HZ).round() as usize {
                    chip8.process_instruction(&pressed_inputs);
                }

                window.request_redraw();
            }
            Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                WindowEvent::CloseRequested => exit(0),
                WindowEvent::RedrawRequested => chip8.display.render(),
                WindowEvent::KeyboardInput {
                    device_id: _,
                    event,
                    is_synthetic: _,
                } => {
                    if let Some(key) = &KEYS.iter().position(|key| *key == event.physical_key) {
                        pressed_inputs[*key] = event.state == ElementState::Pressed;
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .expect("failed to run event loop");
}

const KEYS: [KeyCode; 16] = [
    KeyCode::KeyX,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::KeyQ,
    KeyCode::KeyW,
    KeyCode::KeyE,
    KeyCode::KeyA,
    KeyCode::KeyS,
    KeyCode::KeyD,
    KeyCode::KeyZ,
    KeyCode::KeyC,
    KeyCode::Digit4,
    KeyCode::KeyR,
    KeyCode::KeyF,
    KeyCode::KeyV,
];
