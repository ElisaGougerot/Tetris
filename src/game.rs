extern crate opengl_graphics;
extern crate piston_window;
extern crate rand;

use opengl_graphics::GlGraphics;
use piston_window::*;
use std::time::{Duration, Instant};

use board;

pub struct Falling {
    offset: (isize, isize),
    piece: board::Piece,
    time_since_fall: Instant,
}

/// State of the piece: can be :
/// Falling: The piece is Falling
/// Line: The piece has formed a Line
/// GameOver: The game is over, you lost.
enum State {
    Falling(Falling),
    Line(isize, Instant, Vec<usize>),
    GameOver,
}

pub struct Game {
    board: board::Board,
    metrics: board::Metrics,
    possible_pieces: Vec<board::Board>,
    state: State,
}

/// Initialization of the board.
impl Game {
    pub fn new(metrics: board::Metrics) -> Self {
        let __ = 0;
        let xx = 01;
        let possible_pieces = vec![
            board::Board::piece(
                &[
                    [__, __, __, __],
                    [__, xx, xx, xx],
                    [__, xx, __, __],
                    [__, __, __, __],
                ],
                board::Color::Orange,
            ),
            board::Board::piece(
                &[
                    [__, __, __, __],
                    [__, xx, xx, xx],
                    [__, __, __, xx],
                    [__, __, __, __],
                ],
                board::Color::Yellow,
            ),
            board::Board::piece(
                &[
                    [__, __, __, __],
                    [xx, xx, xx, xx],
                    [__, __, __, __],
                    [__, __, __, __],
                ],
                board::Color::Blue,
            ),
            board::Board::piece(
                &[
                    [xx, xx, xx, __],
                    [__, xx, __, __],
                    [__, __, __, __],
                    [__, __, __, __],
                ],
                board::Color::Green,
            ),
            board::Board::piece(
                &[
                    [__, __, __, __],
                    [__, xx, xx, __],
                    [xx, xx, __, __],
                    [__, __, __, __],
                ],
                board::Color::Cyan,
            ),
            board::Board::piece(
                &[
                    [__, __, __, __],
                    [xx, xx, __, __],
                    [__, xx, xx, __],
                    [__, __, __, __],
                ],
                board::Color::Magenta,
            ),
            board::Board::piece(
                &[
                    [__, __, __, __],
                    [__, xx, xx, __],
                    [__, xx, xx, __],
                    [__, __, __, __],
                ],
                board::Color::Red,
            ),
        ]
        .into_iter()
        .map(|x| x.with_trim_sides())
        .collect();

        Game {
            board: board::Board::empty(metrics.board_x, metrics.board_y),
            state: State::Falling(Self::new_falling(&possible_pieces)),
            possible_pieces,
            metrics,
        }
    }

    /// Function that allows to let a random piece drop from a given position and instantly.
    pub fn new_falling(possible_pieces: &Vec<board::Board>) -> Falling {
        let idx = rand::random::<usize>() % possible_pieces.len();

        Falling {
            offset: (0, 0),
            piece: possible_pieces[idx].clone(),
            time_since_fall: Instant::now(),
        }
    }


    pub fn move_piece(&mut self, change: (isize, isize)) {
        let opt_new_state = match &mut self.state {
            State::GameOver | State::Line(_, _, _) => None,
            State::Falling(falling) => {
                let new_offset = {
                    let (x, y) = falling.offset;
                    ((x as isize + change.0), (y as isize + change.1))
                };
                let is_down = change == (0, 1);

                if self.board.as_merged(new_offset, &falling.piece).is_none() {
                    //==> Collision
                    if is_down {
                        match self.board.as_merged(falling.offset, &falling.piece) {
                            None => Some(State::GameOver),
                            Some(merged_board) => {
                                let completed = merged_board.get_full_lines_indicts();
                                self.board = merged_board;

                                *falling = Self::new_falling(&self.possible_pieces);
                                if completed.len() > 0 {
                                    Some(State::Line(0, Instant::now(), completed))
                                } else {
                                    None
                                }
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    //=> Piece is falling
                    falling.offset = new_offset;
                    if is_down {
                        falling.time_since_fall = Instant::now();
                    }
                    None
                }
            }
        };

        if let Some(new_state) = opt_new_state {
            self.state = new_state;
        }
    }

    /// Rotation of the piece
    pub fn rotate(&mut self, counter: bool) {
        match &mut self.state {
            State::GameOver | State::Line(_, _, _) => {}
            State::Falling(falling) => {
                let rotated_piece = if counter {
                    falling.piece.with_rotated()
                } else {
                    falling.piece.with_rotated_counter()
                };

                self.board
                    .as_merged(falling.offset, &rotated_piece)
                    .map(|_| {
                        falling.piece = rotated_piece;
                    });
            }
        }
    }

    pub fn progress(&mut self) {
        enum Disposition {
            ShouldFall,
            NewPiece(board::Board),
        }

        let disp = match &mut self.state {
            State::GameOver => return,
            State::Line(stage, last_stage_switch, lines) => {
                if last_stage_switch.elapsed() <= Duration::from_millis(50) {
                    return;
                }
                if *stage < 18 {
                    *stage += 1;
                    *last_stage_switch = Instant::now();
                    return;
                } else {
                    Disposition::NewPiece(self.board.with_eliminate_lines(lines))
                }
            }
            State::Falling(falling) => {
                if falling.time_since_fall.elapsed() <= Duration::from_millis(700) {
                    return;
                }
                Disposition::ShouldFall
            }
        };

        match disp {
            Disposition::ShouldFall => self.move_piece((0, 1)),
            Disposition::NewPiece(new_board) => {
                self.board = new_board;
                self.state = State::Falling(Self::new_falling(&self.possible_pieces));
            }
        }
    }

    pub fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let res = self.metrics.resolution();
        let c = &Context::new_abs(res[0] as f64, res[1] as f64);

        gl.draw(args.viewport(), |_, gl| match &self.state {
            State::Line(_stage, _, _) => {
                self.board.draw(c, gl, board::DrawEffect::None, &self.metrics);
            }
            State::Falling(falling) => {
                if let Some(merged) = self.board.as_merged(falling.offset, &falling.piece) {
                    merged.draw(c, gl, board::DrawEffect::None, &self.metrics);
                }
            }
            State::GameOver => {
                self.board.draw(c, gl, board::DrawEffect::Loose, &self.metrics);
            }
        });
    }

    pub fn on_press(&mut self, args: &Button) {
        match args {
            Button::Keyboard(key) => {
                self.on_key(key);
            }
            _ => {}
        }
    }

    fn on_key(&mut self, key: &Key) {
        let movement = match key {
            Key::Right => Some((1, 0)),
            Key::Left => Some((-1, 0)),
            Key::Down => Some((0, 1)),
            _ => None,
        };

        if let Some(movement) = movement {
            self.move_piece(movement);
            return;
        }

        match key {
            Key::Up => self.rotate(false),
            Key::NumPad5 => self.rotate(true),
            _ => return,
        }
    }
}
