#![feature(const_generics)]
#![allow(incomplete_features)]
#![cfg_attr(not(test), no_std)]

use core::ops::Sub;
use pc_keyboard::{DecodedKey, KeyCode};

const UPDATE_FREQUENCY: usize = 3;

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
pub struct SnakeGame<const WIDTH: usize, const HEIGHT: usize> {
    cells: [[Cell; WIDTH]; HEIGHT],
    snake: Snake<WIDTH,HEIGHT>,
    status: Status,
    food_eaten: u32,
    countdown: usize,
    last_key: Option<Dir>
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Dir {
    N, S, E, W
}


impl Dir {
    fn icon(&self) -> char {
        match self {
            Dir::N => 'v',
            Dir::S => '^',
            Dir::E => '<',
            Dir::W => '>'
        }
    }

    fn left(&self) -> Dir {
        match self {
            Dir::N => Dir::W,
            Dir::S => Dir::E,
            Dir::E => Dir::N,
            Dir::W => Dir::S
        }
    }

    fn right(&self) -> Dir {
        match self {
            Dir::N => Dir::E,
            Dir::S => Dir::W,
            Dir::E => Dir::S,
            Dir::W => Dir::N
        }
    }
}

impl From<char> for Dir {
    fn from(icon: char) -> Self {
        match icon {
            '^' => Dir::S,
            'v' => Dir::N,
            '>' => Dir::W,
            '<' => Dir::E,
            _ => panic!("Illegal icon: '{}'", icon)
        }
    }
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
#[repr(u8)]
pub enum Cell {
    Food,
    Empty,
    Wall,
    Body,
}

#[derive(Debug,Copy,Clone,Eq,PartialEq)]
pub struct Position<const WIDTH: usize, const HEIGHT: usize> {
    col: i16, row: i16
}

impl <const WIDTH: usize, const HEIGHT: usize> Sub for Position<WIDTH,HEIGHT> {
    type Output = Position<WIDTH,HEIGHT>;

    fn sub(self, rhs: Self) -> Self::Output {
        Position {col: self.col - rhs.col, row: self.row - rhs.row}
    }
}

impl <const WIDTH: usize, const HEIGHT: usize> Position<WIDTH,HEIGHT> {
    pub fn is_legal(&self) -> bool {
        0 <= self.col && self.col < WIDTH as i16 && 0 <= self.row && self.row < HEIGHT as i16
    }

    pub fn row_col(&self) -> (usize, usize) {
        (self.row as usize, self.col as usize)
    }

    pub fn neighbor(&self, d: Dir) -> Position<WIDTH,HEIGHT> {
        match d {
            Dir::N => Position {row: self.row - 1, col: self.col},
            Dir::S => Position {row: self.row + 1, col: self.col},
            Dir::E => Position {row: self.row,     col: self.col + 1},
            Dir::W => Position {row: self.row,     col: self.col - 1}
        }
    }
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
struct Snake<const WIDTH: usize, const HEIGHT: usize> {
    pos: Position<WIDTH,HEIGHT>, dir: Dir, open: bool
}

impl <const WIDTH: usize, const HEIGHT: usize> Snake<WIDTH,HEIGHT> {
    fn new(pos: Position<WIDTH,HEIGHT>, icon: char) -> Self {
        Snake {pos, dir: Dir::from(icon), open: true}
    }

    fn tick(&mut self) {
        self.open = !self.open;
    }

    fn icon(&self) -> char {
        if self.open {
            self.dir.icon()
        } else {
            match self.dir {
                Dir::N | Dir::S => '|',
                Dir::E | Dir::W => '-'
            }
        }
    }
}

#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum Status {
    Normal,
    Over,
}

const SNAKE_START_DIR: [Dir; 4] = [Dir::E, Dir::W, Dir::E, Dir::W];

const START: &'static str =
    "################################################################################
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                      <                                       #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                  *                                                           #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     #                                                                              #
     ################################################################################";

impl <const WIDTH: usize, const HEIGHT: usize> SnakeGame<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        let mut game = SnakeGame {
            cells: [[Cell::Food; WIDTH]; HEIGHT],
            snake: Snake::new(Position { col: 0, row: 0 }, '>'),
            food_eaten: 0,
            countdown: UPDATE_FREQUENCY,
            last_key: None,
            status: Status::Normal,
        };
        game.reset();
        game
    }

    fn reset(&mut self) {
        for (row, row_chars) in START.split('\n').enumerate() {
            for (col, icon) in row_chars.trim().chars().enumerate() {
                self.translate_icon(row, col, icon);
            }
        }
        self.status = Status::Normal;
        self.food_eaten = 0;
        self.last_key = None;
    }

    pub fn score(&self) -> u32 {
        self.food_eaten
    }

    fn translate_icon(&mut self, row: usize, col: usize, icon: char) {
        match icon {
            '#' => self.cells[row][col] = Cell::Wall,
            '*' => self.cells[row][col] = Cell::Food,
            '>' | '<' | '^' | 'v' => {
                self.snake = Snake::new(Position { row: row as i16, col: col as i16 }, icon);
            },
            _ => panic!("Unrecognized character: '{}'", icon)
        }
    }

    pub fn cell(&self, p: Position<WIDTH, HEIGHT>) -> Cell {
        self.cells[p.row as usize][p.col as usize]
    }

    pub fn cell_pos_iter(&self) -> RowColIter<WIDTH, HEIGHT> {
        RowColIter { row: 0, col: 0 }
    }

    pub fn snake_at(&self) -> Position<WIDTH, HEIGHT> {
        self.snake.pos
    }

    pub fn snake_icon(&self) -> char {
        self.snake.icon()
    }

    pub fn update(&mut self) {
        self.resolve_move();
        self.last_key = None;
        self.snake.tick();
    }

    fn ahead_left_right(&self, p: Position<WIDTH, HEIGHT>, dir: Dir) -> (Cell, Cell, Cell) {
        let ahead = self.cell(p.neighbor(dir));
        let left = self.cell(p.neighbor(dir.left()));
        let right = self.cell(p.neighbor(dir.right()));
        (ahead, left, right)
    }

    pub fn countdown_complete(&mut self) -> bool {
        if self.countdown == 0 {
            self.countdown = UPDATE_FREQUENCY;
            true
        } else {
            self.countdown -= 1;
            false
        }
    }

    pub fn key(&mut self, key: DecodedKey) {
        match self.status {
            Status::Over => {
                match key {
                    DecodedKey::RawKey(KeyCode::S) | DecodedKey::Unicode('s') => self.reset(),
                    _ => {}
                }
            }
            _ => {
                let key = match key {
                    DecodedKey::RawKey(k) => match k {
                        KeyCode::ArrowUp => Some(Dir::N),
                        KeyCode::ArrowDown => Some(Dir::S),
                        KeyCode::ArrowLeft => Some(Dir::W),
                        KeyCode::ArrowRight => Some(Dir::E),
                        _ => None
                    }
                    DecodedKey::Unicode(c) => match c {
                        'w' => Some(Dir::N),
                        'a' => Some(Dir::W),
                        's' => Some(Dir::S),
                        'd' => Some(Dir::E),
                        _ => None
                    }
                };
                if key.is_some() {
                    self.last_key = key;
                }
            }
        }
    }

    fn resolve_move(&mut self) {
        if let Some(dir) = self.last_key {
            let neighbor = self.snake.pos.neighbor(dir);
            if neighbor.is_legal() {
                let (row, col) = neighbor.row_col();
                if self.cells[row][col] != Cell::Wall {
                    self.move_to(neighbor, dir);
                }
            }
        }
    }

    fn move_to(&mut self, neighbor: Position<WIDTH, HEIGHT>, dir: Dir) {
        self.snake.pos = neighbor;
        self.snake.dir = dir;
        let (row, col) = neighbor.row_col();
        match self.cells[row][col] {
            Cell::Food => {
                self.food_eaten += 1;
                self.cells[row][col] = Cell::Empty;
            }

            _ => {}
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    fn key2dir(key: DecodedKey) -> Option<Dir> {
        match key {
            DecodedKey::RawKey(k) => match k {
                KeyCode::ArrowUp => Some(Dir::N),
                KeyCode::ArrowDown => Some(Dir::S),
                KeyCode::ArrowLeft => Some(Dir::W),
                KeyCode::ArrowRight => Some(Dir::E),
                _ => None
            }
            DecodedKey::Unicode(c) => match c {
                'w' => Some(Dir::N),
                'a' => Some(Dir::W),
                's' => Some(Dir::S),
                'd' => Some(Dir::E),
                _ => None
            }
        }

    }
}


    pub struct RowColIter<const WIDTH: usize, const HEIGHT: usize> {
        row: usize, col: usize
    }

    impl <const WIDTH: usize, const HEIGHT: usize> Iterator for RowColIter<WIDTH,HEIGHT> {
        type Item = Position<WIDTH,HEIGHT>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.row < HEIGHT {
                let result = Some(Position {row: self.row as i16, col: self.col as i16});
                self.col += 1;
                if self.col == WIDTH {
                    self.col = 0;
                    self.row += 1;
                }
                result
            } else {
                None
            }
        }
    }




