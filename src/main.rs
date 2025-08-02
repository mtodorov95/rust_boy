use std::{
    env,
    time::{Duration, Instant},
};

use sdl2::{keyboard::Keycode, pixels::Color};

const PIXEL_SIZE: u32 = 4;
const WIDTH: u32 = 160 * PIXEL_SIZE;
const HEIGHT: u32 = 144 * PIXEL_SIZE;

mod rust_boy;

fn main() {
    let sdl_ctx = sdl2::init().unwrap();
    let video_subsystem = sdl_ctx.video().unwrap();

    let window = video_subsystem
        .window("rust_boy", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_ctx.event_pump().unwrap();
    let mut running: bool = true;

    // Args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: rust_boy <path_to_rom>");
        return;
    }
    while running {
        let start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    println!("Exiting...");
                    running = false;
                }
                _ => {}
            }
        }

        // TODO: Cycle
        // TODO: Draw display
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::GREEN);

        canvas.present();

        let delay = 2000;
        let elapsed = start.elapsed();
        if elapsed < Duration::from_micros(delay) {
            std::thread::sleep(Duration::from_micros(delay) - elapsed);
        }
    }
}
