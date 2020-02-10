extern crate opengl_graphics;
extern crate piston_window;
extern crate rand;

use opengl_graphics::GlGraphics;
use piston_window::*;

mod game;
mod board;

fn main() {
    let metrics = board::Metrics {
        block_pixels: 25,
        board_x: 12,
        board_y: 25,
    };

    let mut window: PistonWindow = WindowSettings::new("Tetris", metrics.resolution())
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed: {}", e));

    let mut gl = GlGraphics::new(OpenGL::V3_2);
    let mut game = game::Game::new(metrics);

    while let Some(e) = window.next() {
        game.progress();

        if let Some(args) = e.render_args() {
            game.render(&mut gl, &args);
        }

        if let Some(args) = e.press_args() {
            game.on_press(&args);
        }
    }
}
