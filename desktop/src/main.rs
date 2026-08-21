//! CHIP-8 SDL2 frontend and runner.
//!
//! Provides the window, canvas rendering, keyboard event pump, and main game loop
//! interface for the `chip8_core` emulation engine.

use chip8_core::*;

use std::env;
use std::fs::File;
use std::io::Read;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// Upscaling factor applied to each native 1x1 CHIP-8 pixel on the SDL window.
const SCALE: u32 = 15;

/// Derived window width in physical display pixels.
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;

/// Derived window height in physical display pixels.
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

/// Number of CPU instruction cycles executed per rendered frame (~600 Hz CPU clock).
const TICKS_PER_FRAME: usize = 10;

/// Application entry point. Initializes SDL2, loads the target ROM, and drives
/// the main emulation cycle.
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    // Initialize SDL2 context and window canvas
    let sdl_context = sdl2::init().expect("Failed to initialize SDL2 context");
    let video_subsystem = sdl_context.video().expect("Failed to initialize video subsystem");
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .expect("Failed to build SDL window");

    let mut canvas = window.into_canvas().present_vsync().build().expect("Failed to build canvas");
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().expect("Failed to create event pump");
    let mut chip8 = Emulator::new();

    // Load ROM file into memory
    let mut rom = File::open(&args[1]).expect("Unable to open ROM file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).expect("Failed to read ROM file into buffer");
    chip8.load(&buffer);

    // Primary emulation and rendering loop
    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, false);
                    }
                }
                _ => (),
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }
        chip8.tick_timer();
        draw_screen(&chip8, &mut canvas);
    }
}

/// Renders the emulator's 1D framebuffer onto the scaled SDL2 window canvas.
///
/// # Arguments
/// * `emu` - Reference to the active emulator state.
/// * `canvas` - Mutable reference to the SDL2 drawing canvas.
fn draw_screen(emu: &Emulator, canvas: &mut Canvas<Window>) {
    // Clear screen to black background
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // Draw active pixels as white scaled rectangles
    let screen_buf = emu.get_display();
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

/// Maps standard QWERTY keyboard inputs to standard hexadecimal CHIP-8 keypad indices.
///
/// ```text
/// Keyboard Layout              CHIP-8 Hex Keypad
/// +---+---+---+---+            +---+---+---+---+
/// | 1 | 2 | 3 | 4 |            | 1 | 2 | 3 | C |
/// +---+---+---+---+            +---+---+---+---+
/// | Q | W | E | R |            | 4 | 5 | 6 | D |
/// +---+---+---+---+    ==>     +---+---+---+---+
/// | A | S | D | F |            | 7 | 8 | 9 | E |
/// +---+---+---+---+            +---+---+---+---+
/// | Z | X | C | V |            | A | 0 | B | F |
/// +---+---+---+---+            +---+---+---+---+
/// ```
///
/// # Arguments
/// * `key` - SDL2 `Keycode` received from the event loop.
///
/// # Returns
/// An `Option<usize>` containing the keypad value (`0x0..=0xF`) if matched.
fn key2btn(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
