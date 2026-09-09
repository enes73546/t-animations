use std::env;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H\x1b[?25l";

const ANSI_COLORS: &[&str] = &[
    "\x1b[1;31m",
    "\x1b[1;32m", 
    "\x1b[1;33m", 
    "\x1b[1;34m", 
    "\x1b[1;35m", 
    "\x1b[1;36m",
    "\x1b[1;37m",
];

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn delta(self) -> (i16, i16) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

struct Pipe {
    x: i16,
    y: i16,
    dir: Direction,
    color: &'static str,
}

struct SimpleRng(u64);

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        SimpleRng(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }

    fn range(&mut self, max: u64) -> u64 {
        if max == 0 { 0 } else { self.next() % max }
    }
}

fn get_turn_symbol(old_dir: Direction, new_dir: Direction) -> char {
    match (old_dir, new_dir) {
        (Direction::Up, Direction::Right) | (Direction::Left, Direction::Down) => '╔',
        (Direction::Up, Direction::Left) | (Direction::Right, Direction::Down) => '╗',
        (Direction::Down, Direction::Right) | (Direction::Left, Direction::Up) => '╚',
        (Direction::Down, Direction::Left) | (Direction::Right, Direction::Up) => '╝',
        _ => '╬',
    }
}

fn get_straight_symbol(dir: Direction) -> char {
    match dir {
        Direction::Up | Direction::Down => '║',
        Direction::Left | Direction::Right => '═',
    }
}

fn create_random_pipe(width: i16, height: i16, rng: &mut SimpleRng) -> Pipe {
    let color = ANSI_COLORS[rng.range(ANSI_COLORS.len() as u64) as usize];
    let side = rng.range(4);

    let (x, y, dir) = match side {
        0 => (rng.range(width as u64) as i16, 0, Direction::Down),
        1 => (width - 1, rng.range(height as u64) as i16, Direction::Left),
        2 => (rng.range(width as u64) as i16, height - 1, Direction::Up),
        _ => (0, rng.range(height as u64) as i16, Direction::Right),
    };

    Pipe { x, y, dir, color }
}

fn run_pipes(speed: u64) {
    let mut rng = SimpleRng::new();
    let width = 100i16;
    let height = 30i16;

    let mut grid = vec![vec![' '; width as usize]; height as usize];
    let mut active_pipes: Vec<Pipe> = Vec::new();

    print!("{}", CLEAR_SCREEN);
    io::stdout().flush().unwrap();

    let frame_delay = Duration::from_millis(60 / speed.clamp(1, 5));

    loop {
        if rng.range(4) == 0 || active_pipes.is_empty() {
            if active_pipes.len() < 10 {
                active_pipes.push(create_random_pipe(width, height, &mut rng));
            }
        }

        let mut i = 0;
        while i < active_pipes.len() {
            let pipe = &mut active_pipes[i];

            if pipe.x < 0 || pipe.x >= width || pipe.y < 0 || pipe.y >= height {
                active_pipes.remove(i);
                continue;
            }

            let ux = pipe.x as usize;
            let uy = pipe.y as usize;

            if grid[uy][ux] != ' ' {
                active_pipes.remove(i);
                continue;
            }

            let is_turning = rng.range(5) == 0;
            let next_dir = if is_turning {
                match pipe.dir {
                    Direction::Up | Direction::Down => {
                        if rng.range(2) == 0 { Direction::Left } else { Direction::Right }
                    }
                    Direction::Left | Direction::Right => {
                        if rng.range(2) == 0 { Direction::Up } else { Direction::Down }
                    }
                }
            } else {
                pipe.dir
            };

            let symbol = if is_turning {
                get_turn_symbol(pipe.dir, next_dir)
            } else {
                get_straight_symbol(pipe.dir)
            };

            grid[uy][ux] = symbol;

            print!(
                "\x1b[{};{}H{}{}\x1b[0m",
                pipe.y + 1,
                pipe.x + 1,
                pipe.color,
                symbol
            );
            pipe.dir = next_dir;
            let (dx, dy) = pipe.dir.delta();
            pipe.x += dx;
            pipe.y += dy;

            i += 1;
        }

        io::stdout().flush().unwrap();
        thread::sleep(frame_delay);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || (args[1] != "0001" && args[1].to_lowercase() != "pipes") {
        println!("[ \x1b[1;31mERROR\x1b[0m ] Usage: cargo run -- pipes [--speed 1-5]");
        return;
    }

    let mut speed = 1;
    if args.len() >= 4 && args[2] == "--speed" {
        speed = args[3].parse::<u64>().unwrap_or(1);
    }

    run_pipes(speed);
}