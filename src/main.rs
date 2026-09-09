use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H\x1b[?25l";
const RESET: &str = "\x1b[0m";

const COLORS: &[&str] = &[
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

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    color: &'static str,
}

struct Pipe {
    x: i16,
    y: i16,
    dir: Direction,
    color: &'static str,
}

struct FastRng(u64);

impl FastRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as u64;
        FastRng(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }

    fn range(&mut self, max: u64) -> u64 {
        if max == 0 { 0 } else { self.next() % max }
    }
}

fn terminal_size() -> (i16, i16) {
    let output = Command::new("cmd")
        .args(["/C", "mode", "con"])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        let mut width = 120;
        let mut height = 40;

        for line in text.lines() {
            if line.contains("Columns:") {
                if let Some(val) = line.split(':').nth(1) {
                    width = val.trim().parse().unwrap_or(120);
                }
            }
            if line.contains("Lines:") {
                if let Some(val) = line.split(':').nth(1) {
                    height = val.trim().parse().unwrap_or(40);
                }
            }
        }
        (width.max(10), height.max(5))
    } else {
        (120, 40)
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

fn create_random_pipe(width: i16, height: i16, rng: &mut FastRng) -> Pipe {
    let color = COLORS[rng.range(COLORS.len() as u64) as usize];
    let side = rng.range(4);

    let (x, y, dir) = match side {
        0 => (rng.range(width as u64) as i16, 0, Direction::Down),
        1 => (width - 1, rng.range(height as u64) as i16, Direction::Left),
        2 => (rng.range(width as u64) as i16, height - 1, Direction::Up),
        _ => (0, rng.range(height as u64) as i16, Direction::Right),
    };

    Pipe { x, y, dir, color }
}

fn pipes(speed: u64, cross_chance: u64) {
    let mut rng = FastRng::new();
    let mut size = terminal_size();
    let empty_cell = Cell { ch: ' ', color: RESET };
    let mut grid = vec![vec![empty_cell; size.0 as usize]; size.1 as usize];
    let mut active_pipes: Vec<Pipe> = Vec::new();

    print!("{}", CLEAR_SCREEN);
    io::stdout().flush().unwrap();

    let delay = Duration::from_millis(45 / speed.clamp(1, 5));

    loop {
        let new_size = terminal_size();
        if new_size != size {
            size = new_size;
            grid = vec![vec![empty_cell; size.0 as usize]; size.1 as usize];
            print!("{}", CLEAR_SCREEN);
            active_pipes.clear();
        }

        let (width, height) = size;

        if rng.range(3) == 0 || active_pipes.is_empty() {
            if active_pipes.len() < 25 {
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
            let current_cell = grid[uy][ux];

            let symbol;
            let mut next_dir = pipe.dir;

            if current_cell.ch != ' ' {
                if current_cell.color == pipe.color {
                    active_pipes.remove(i);
                    continue;
                }

                let can_cross = cross_chance == 100 || (cross_chance > 0 && rng.range(100) < cross_chance);

                if current_cell.ch != '╬' && can_cross {
                    symbol = '╬';
                } else {
                    active_pipes.remove(i);
                    continue;
                }
            } else {
                let is_turning = rng.range(6) == 0;
                if is_turning {
                    next_dir = match pipe.dir {
                        Direction::Up | Direction::Down => {
                            if rng.range(2) == 0 { Direction::Left } else { Direction::Right }
                        }
                        Direction::Left | Direction::Right => {
                            if rng.range(2) == 0 { Direction::Up } else { Direction::Down }
                        }
                    };
                    symbol = get_turn_symbol(pipe.dir, next_dir);
                } else {
                    symbol = get_straight_symbol(pipe.dir);
                }
            }

            grid[uy][ux] = Cell { ch: symbol, color: pipe.color };

            print!(
                "\x1b[{};{}H{}{}{}",
                pipe.y + 1,
                pipe.x + 1,
                pipe.color,
                symbol,
                RESET
            );

            pipe.dir = next_dir;
            let (dx, dy) = pipe.dir.delta();
            pipe.x += dx;
            pipe.y += dy;

            i += 1;
        }

        io::stdout().flush().unwrap();
        thread::sleep(delay);
    }
}

fn invalid_speed_err() {
    println!("[ \x1b[1;31mERROR\x1b[0m ] Speed Must Be Between 1x And 5x");
}

fn invalid_chance_err() {
    println!("[ \x1b[1;31mERROR\x1b[0m ] Chance Must Be Between 1 And 100");
}

fn invalid_animation_err() {
    println!("[ \x1b[1;31mERROR\x1b[0m ] Invalid Animation Please Run With Valid Arguments");
}

fn insufficient_args_err() {
    println!("[ \x1b[1;31mERROR\x1b[0m ] Insufficient Arguments Please Enter The Animation ID or Name");
}

fn load_animation(args: &[String]) {
    if args[0] == "0001" || args[0].to_lowercase() == "pipes" {
        let mut speed = 1;
        let mut chance = 12;

        let mut idx = 1;
        while idx < args.len() {
            if args[idx] == "--speed" {
                if idx + 1 >= args.len() {
                    invalid_speed_err();
                    return;
                }
                match args[idx + 1].parse::<u64>() {
                    Ok(value) if (1..=5).contains(&value) => {
                        speed = value;
                        idx += 2;
                    }
                    _ => {
                        invalid_speed_err();
                        return;
                    }
                }
            } else if args[idx] == "--chance" || args[idx] == "--change" {
                if idx + 1 >= args.len() {
                    invalid_chance_err();
                    return;
                }
                match args[idx + 1].parse::<u64>() {
                    Ok(value) if (1..=100).contains(&value) => {
                        chance = value;
                        idx += 2;
                    }
                    _ => {
                        invalid_chance_err();
                        return;
                    }
                }
            } else {
                invalid_animation_err();
                return;
            }
        }

        pipes(speed, chance);
    } else {
        invalid_animation_err();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        insufficient_args_err();
        return;
    }

    load_animation(&args[1..]);
}