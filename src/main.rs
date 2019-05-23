use ggez::conf;
use ggez::event;
use ggez::event::{KeyCode, KeyMods};
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
use na::{Point2, Vector2};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::ops::Neg;
use std::time::{Duration, Instant};

const GRID_SIZE: (i32, i32) = (16, 32);
const GRID_CELL_SIZE: (i32, i32) = (32, 32);

const SCREEN_SIZE: (i32, i32) = (
    GRID_SIZE.0 * GRID_CELL_SIZE.0,
    GRID_SIZE.1 * GRID_CELL_SIZE.1,
);

const UPDATES_PER_SECOND: f32 = 10.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

struct MainState {
    pos: na::Point2<i32>,
    facing: u8,
    tetromino: Tetromino,
    start_time: Instant,
    updates_so_far: i32,
    board: Board,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

struct Board {
    data: [[Option<FixedBlock>; 16]; 32],
}

impl Board {
    fn get(&self, block: na::Point2<i32>) -> Option<&Option<FixedBlock>> {
        match self.data.get(block[1] as usize) {
            None => None,
            Some(row) => row.get(block[0] as usize),
        }
    }

    fn get_mut(&mut self, block: na::Point2<i32>) -> Option<&mut Option<FixedBlock>> {
        match self.data.get_mut(block[1] as usize) {
            None => None,
            Some(row) => row.get_mut(block[0] as usize),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FixedBlock {
    tetromino: Tetromino,
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            pos: na::Point2::new(rand::thread_rng().gen_range(0, 15), 0),
            facing: 0,
            start_time: Instant::now(),
            tetromino: rand::random(),
            updates_so_far: 0,
            board: Board {
                data: [[None; 16]; 32],
            },
            min_x: 0,
            max_x: 0,
            min_y: 0,
            max_y: 0,
        };
        s.min_x = s.tetromino.min_x(s.facing);
        s.max_x = s.tetromino.max_x(s.facing);
        s.min_y = s.tetromino.min_y(s.facing);
        s.max_y = s.tetromino.max_y(s.facing);
        Ok(s)
    }
    fn not_overlapping_down(&self) -> bool {
        self.tetromino
            .blocks(self.pos + na::Vector2::new(0, 1), self.facing)
            .into_iter()
            .all(|block| self.board.get(block) == Some(&None))
    }
    fn not_overlapping_left(&self) -> bool {
        self.tetromino
            .blocks(self.pos + na::Vector2::new(-1, 0), self.facing)
            .into_iter()
            .all(|block| self.board.get(block) == Some(&None))
    }
    fn not_overlapping_right(&self) -> bool {
        self.tetromino
            .blocks(self.pos + na::Vector2::new(1, 0), self.facing)
            .into_iter()
            .all(|block| self.board.get(block) == Some(&None))
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if Instant::now() - self.start_time
            >= Duration::from_millis(MILLIS_PER_UPDATE * self.updates_so_far as u64)
        {
            if self.not_overlapping_down() {
                self.pos[1] += 1;
            } else {
                let fixed_block = FixedBlock {
                    tetromino: self.tetromino,
                };
                for block in self.tetromino.blocks(self.pos, self.facing) {
                    match self.board.get_mut(block) {
                        Some(ref mut a) if a.is_none() => **a = Some(fixed_block),
                        _ => panic!("{:?}", block),
                    }
                }
                self.tetromino = rand::random();
                self.facing = rand::thread_rng().gen_range(0, 4);

                self.min_x = self.tetromino.min_x(self.facing);
                self.max_x = self.tetromino.max_x(self.facing);

                self.pos[0] = rand::thread_rng().gen_range(-self.min_x, 16 - self.max_x);

                self.min_y = self.tetromino.min_y(self.facing);
                self.pos[1] = -self.min_y;
            }
            self.updates_so_far += 1;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let blocks = self.tetromino.blocks(self.pos, self.facing);

        for block in blocks {
            let rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                block_rect(block),
                self.tetromino.color(),
            )?;
            graphics::draw(ctx, &rectangle, (na::Point2::new(0.0, 0.0),))?;
        }

        // Draw fixed blocks.
        for (y, row) in self.board.data.iter().enumerate() {
            for (x, square) in row.iter().enumerate() {
                let block = na::Point2::new(x as i32, y as i32);
                if let Some(b) = square {
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        block_rect(block),
                        b.tetromino.color(),
                    )?;
                    graphics::draw(ctx, &rectangle, (na::Point2::new(0.0, 0.0),))?;
                };
            }
        }
        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Left => {
                if self.pos[0] > -self.min_x && self.not_overlapping_left() {
                    self.pos[0] -= 1
                }
            }
            KeyCode::Right => {
                if self.pos[0] < (15 - self.max_x) && self.not_overlapping_right() {
                    self.pos[0] += 1
                }
            }
            KeyCode::Up => {
                if self.not_overlapping_left()
                    && self.not_overlapping_right()
                    && self.not_overlapping_down()
                {
                    self.facing += 1
                }
            }
            KeyCode::Down => {
                if self.pos[1] < 15 - self.max_y {
                    self.pos[1] += 1
                }
            }
            KeyCode::Space => {
                while self.not_overlapping_down() {
                    self.pos[1] += 1
                }
            }
            _ => (),
        }
    }
}

pub trait Rotate90 {
    fn rotate_90(self, facing: u8) -> Self;
}

impl<T: na::Scalar + Neg<Output = T>> Rotate90 for Vector2<T> {
    fn rotate_90(self, facing: u8) -> Self {
        match facing % 4 {
            0 => self,
            1 => Vector2::new(-self[1], self[0]),
            2 => Vector2::new(-self[0], -self[1]),
            3 => Vector2::new(self[1], -self[0]),
            _ => unreachable!(),
        }
    }
}

fn block_rect(block: Point2<i32>) -> graphics::Rect {
    graphics::Rect::new(
        GRID_CELL_SIZE.0 as f32 * block[0] as f32,
        GRID_CELL_SIZE.1 as f32 * block[1] as f32,
        GRID_CELL_SIZE.0 as f32,
        GRID_CELL_SIZE.1 as f32,
    )
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Tetromino {
    IBlock,
    OBlock,
    TBlock,
    SBlock,
    ZBlock,
    JBlock,
    LBlock,
}

impl Tetromino {
    fn block_offsets(self) -> Vec<Vector2<i32>> {
        match self {
            Tetromino::IBlock => vec![
                Vector2::new(0, 0),
                Vector2::new(1, 0),
                Vector2::new(2, 0),
                Vector2::new(-1, 0),
            ],
            Tetromino::OBlock => vec![
                Vector2::new(0, 0),
                Vector2::new(1, 0),
                Vector2::new(0, 1),
                Vector2::new(1, 1),
            ],
            Tetromino::TBlock => vec![
                Vector2::new(0, 0),
                Vector2::new(0, 1),
                Vector2::new(-1, 0),
                Vector2::new(1, 0),
            ],
            Tetromino::SBlock => vec![
                Vector2::new(0, 0),
                Vector2::new(0, 1),
                Vector2::new(1, 0),
                Vector2::new(-1, 1),
            ],
            Tetromino::ZBlock => vec![
                Vector2::new(0, 0),
                Vector2::new(0, 1),
                Vector2::new(-1, 0),
                Vector2::new(1, 1),
            ],
            Tetromino::LBlock => vec![
                Vector2::new(0, 0),
                Vector2::new(0, 1),
                Vector2::new(0, -1),
                Vector2::new(-1, -1),
            ],
            Tetromino::JBlock => vec![
                Vector2::new(0, 0),
                Vector2::new(0, 1),
                Vector2::new(0, -1),
                Vector2::new(1, -1),
            ],
        }
    }

    fn blocks(self, pos: Point2<i32>, facing: u8) -> Vec<Point2<i32>> {
        self.block_offsets()
            .iter()
            .map(|block_vector| pos + block_vector.rotate_90(facing))
            .collect()
    }
    fn color(self) -> graphics::Color {
        match self {
            Tetromino::IBlock => graphics::Color::from_rgb(66, 241, 244),
            Tetromino::OBlock => graphics::Color::from_rgb(233, 237, 42),
            Tetromino::TBlock => graphics::Color::from_rgb(182, 42, 237),
            Tetromino::SBlock => graphics::Color::from_rgb(88, 237, 42),
            Tetromino::ZBlock => graphics::Color::from_rgb(226, 50, 27),
            Tetromino::JBlock => graphics::Color::from_rgb(22, 75, 221),
            Tetromino::LBlock => graphics::Color::from_rgb(219, 108, 17),
        }
    }
    fn min_x(self, facing: u8) -> i32 {
        self.blocks(na::Point2::new(0, 0), facing)
            .into_iter()
            .map(|block| block[0])
            .min()
            .unwrap()
    }
    fn max_x(self, facing: u8) -> i32 {
        self.blocks(na::Point2::new(0, 0), facing)
            .into_iter()
            .map(|block| block[0])
            .max()
            .unwrap()
    }
    fn min_y(self, facing: u8) -> i32 {
        self.blocks(na::Point2::new(0, 0), facing)
            .into_iter()
            .map(|block| block[1])
            .min()
            .unwrap()
    }
    fn max_y(self, facing: u8) -> i32 {
        self.blocks(na::Point2::new(0, 0), facing)
            .into_iter()
            .map(|block| block[1])
            .max()
            .unwrap()
    }
}

impl Distribution<Tetromino> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Tetromino {
        match rng.gen_range(0, 7) {
            0 => Tetromino::IBlock,
            1 => Tetromino::OBlock,
            2 => Tetromino::TBlock,
            3 => Tetromino::SBlock,
            4 => Tetromino::ZBlock,
            5 => Tetromino::JBlock,
            6 => Tetromino::LBlock,
            _ => unreachable!(),
        }
    }
}

pub fn main() -> GameResult {
    let (ctx, events_loop) = &mut ggez::ContextBuilder::new("Tetris?", "Sam")
        .window_setup(conf::WindowSetup::default().title("Tetris?"))
        .window_mode(
            conf::WindowMode::default().dimensions(SCREEN_SIZE.0 as f32, SCREEN_SIZE.1 as f32),
        )
        .build()?;

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, events_loop, state)
}
