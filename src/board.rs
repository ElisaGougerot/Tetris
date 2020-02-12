extern crate opengl_graphics;
extern crate piston_window;
extern crate rand;

use opengl_graphics::GlGraphics;
use piston_window::*;

#[derive(Copy, Clone)]
pub enum Color {
    Red,
    Green,
    Blue,
    Magenta,
    Cyan,
    Yellow,
    Orange,
}

/// Screen Params
pub struct Metrics {
    pub block_pixels: usize,
    pub board_x: usize,
    pub board_y: usize,
}

impl Metrics {
    pub fn resolution(&self) -> [u32; 2] {
        [
            (self.board_x * self.block_pixels) as u32,
            (self.board_y * self.block_pixels) as u32,
        ]
    }
}

type Cell = Option<Color>;

#[derive(Clone)]
pub struct Board {
    cells: Vec<Vec<Cell>>,
}

pub type Piece = Board;

pub enum DrawEffect {
    None,
    Loose,
}

impl Board {
    /// Init Board
    pub fn empty(dim_x: usize, dim_y: usize) -> Self {
        let line: Vec<_> = (0..dim_x).map(|_| None).collect();
        let cells: Vec<_> = (0..dim_y).map(|_| line.clone()).collect();
        Board { cells }
    }

    /// Colision
    pub fn valid(&self, offset: (isize, isize)) -> bool {
        if offset.0 >= 0 && offset.0 < self.dim_x() as isize {
            if offset.1 >= 0 && offset.1 < self.dim_y() as isize {
                return true;
            }
        }

        return false;
    }

    pub fn dim_x(&self) -> usize {
        self.cells[0].len()
    }
    pub fn dim_y(&self) -> usize {
        self.cells.len()
    }

    /// Add Piece in Board
    pub fn piece(spec: &[[u8; 4]; 4], color: Color) -> Self {
        let mut board = Board::empty(spec[0].len(), spec.len());

        for x in 0..spec[0].len() {
            for y in 0..spec.len() {
                board.cells[y][x] = if spec[y][x] != 0 { Some(color) } else { None }
            }
        }

        board
    }

    /// Merge Board with new piece
    pub fn as_merged(&self, offset: (isize, isize), board: &Board) -> Option<Board> {
        let mut copy = self.clone();

        for x in 0..board.dim_x() {
            for y in 0..board.dim_y() {
                let cell = board.cells[y][x];
                if cell.is_some() {
                    let x = x as isize + offset.0;
                    let y = y as isize + offset.1;
                    if !self.valid((x, y)) {
                        return None;
                    }
                    if self.cells[y as usize][x as usize].is_none() {
                        copy.cells[y as usize][x as usize] = cell.clone();
                    } else {
                        // Collision
                        return None;
                    }
                }
            }
        }

        Some(copy)
    }

    /// Draw the Board On Window
    pub fn draw(&self, c: &Context, gl: &mut GlGraphics, effect: DrawEffect, metrics: &Metrics) {
        let mut draw = |color, rect: [f64; 4]| {
            Rectangle::new(color).draw(rect, &DrawState::default(), c.transform, gl);
        };

        for x in 0..self.dim_x() {
            for y in 0..self.dim_y() {
                let block_pixels = metrics.block_pixels as f64;
                let border_size = block_pixels / 20.0;
                let outer = [
                    block_pixels * (x as f64),
                    block_pixels * (y as f64),
                    block_pixels,
                    block_pixels,
                ];
                let inner = [
                    outer[0] + border_size,
                    outer[1] + border_size,
                    outer[2] - border_size * 2.0,
                    outer[3] - border_size * 2.0,
                ];

                draw([0.2, 0.2, 0.2, 1.0], outer);
                draw([0.1, 0.1, 0.1, 1.0], inner);

                self.cells[y][x].map(|color| {
                    let code = match color {
                        Color::Red => [1.0, 0.0, 0.0, 1.0],
                        Color::Green => [0.0, 1.0, 0.0, 1.0],
                        Color::Blue => [0.5, 0.5, 1.0, 1.0],
                        Color::Magenta => [1.0, 0.0, 1.0, 1.0],
                        Color::Cyan => [0.0, 1.0, 1.0, 1.0],
                        Color::Yellow => [1.0, 1.0, 0.0, 1.0],
                        Color::Orange => [1.0, 0.5, 0.0, 1.0],
                    };

                    draw(code, outer);

                    let code = [code[0] * 0.8, code[1] * 0.8, code[2] * 0.8, code[3]];

                    draw(code, inner);
                });

                match effect {
                    DrawEffect::None => {}
                    DrawEffect::Loose => {
                        draw([0.0, 0.0, 0.0, 0.9], outer);
                    }
                }
            }
        }
    }

    /// Remove a line
    pub fn without_line(&self, idx: usize) -> Self {
        let mut board = self.clone();

        board.cells.remove(idx);
        board
    }

    /// Add Empty Line
    pub fn prepend_empty_line(&self) -> Self {
        let line: Vec<_> = (0..self.dim_x()).map(|_| None).collect();
        let mut board = self.clone();

        board.cells.insert(0, line);
        board
    }

    /// Erase Board
    pub fn with_eliminate_lines(&self, lines: &Vec<usize>) -> Self {
        let mut board = self.clone();

        for idx in lines {
            board = board.without_line(*idx);
        }

        for _ in 0..lines.len() {
            board = board.prepend_empty_line();
        }

        board
    }

    /// Create new Board without First and Last Line
    pub fn with_trimmed_lines(&self) -> Self {
        let mut board = self.clone();

        while board.cells[0].iter().all(Cell::is_none) {
            board = board.without_line(0);
        }

        while board.cells[board.dim_y() - 1].iter().all(Cell::is_none) {
            board = board.without_line(board.dim_y() - 1);
        }

        board
    }

    /// Return Complete Line
    pub fn get_full_lines_indicts(&self) -> Vec<usize> {
        self.cells
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, line)| line.iter().all(|cell| !cell.is_none()))
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Transposed Board
    pub fn transposed(&self) -> Self {
        let mut board = Self::empty(self.dim_y(), self.dim_x());

        for x in 0..self.dim_x() {
            for y in 0..self.dim_y() {
                board.cells[x][y] = self.cells[y][x];
            }
        }

        board
    }

    /// Return new Board with sysmetric on vertical axis
    pub fn with_mirrored_y(&self) -> Self {
        let mut board = Self::empty(self.dim_x(), self.dim_y());

        for x in 0..self.dim_x() {
            for y in 0..self.dim_y() {
                board.cells[y][x] = self.cells[y][self.dim_x() - x - 1];
            }
        }

        board
    }

    /// Rotate a piece
    pub fn with_rotated_counter(&self) -> Self {
        self.transposed().with_mirrored_y()
    }

    /// Rotate a piece
    pub fn with_rotated(&self) -> Self {
        self.with_mirrored_y().transposed()
    }

    pub fn with_trim_sides(&self) -> Self {
        self.with_trimmed_lines()
            .transposed()
            .with_trimmed_lines()
            .transposed()
    }
}
