use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::time::Duration;

const RESET: &str = "\x1b[0m";
const B_RED: &str = "\x1b[1;31m";
const B_GREEN: &str = "\x1b[1;32m";
const B_YELLOW: &str = "\x1b[1;33m";
const B_BLUE: &str = "\x1b[1;34m";
const B_MAGENTA: &str = "\x1b[1;35m";
const B_CYAN: &str = "\x1b[1;36m";
const B_WHITE: &str = "\x1b[1;37m";

struct Pipe {
    x: i16,
    y: i16,
    dx: i16,
    dy: i16,
    color: &'static str,
}

fn terminal_size() -> (i16, i16) {
    let output = Command::new("cmd")
        .args(["/C", "mode", "con"])
        .output()
        .unwrap();

    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stdout);
    let mut width = 120;
    let mut height = 40;

    for line in text.lines() {
        if line.contains("Columns:") {
            if let Some(value) = line.split(':').nth(1) {
                width = value.trim().parse().unwrap_or(120);
            }
        }

        if line.contains("Lines:") {
            if let Some(value) = line.split(':').nth(1) {
                height = value.trim().parse().unwrap_or(40);
            }
        }
    }

    (width.max(10), height.max(5))
}

fn random_num(max: u64) -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64 % max
}

fn random_pipe(width: i16, height: i16, color: &'static str) -> Pipe {
    match random_num(4) {
        0 => Pipe {
            x: random_num(width as u64) as i16,
            y: 0,
            dx: 0,
            dy: 1,
            color,
        },

        1 => Pipe {
            x: width - 1,
            y: random_num(height as u64) as i16,
            dx: -1,
            dy: 0,
            color,
        },

        2 => Pipe {
            x: random_num(width as u64) as i16,
            y: height - 1,
            dx: 0,
            dy: -1,
            color,
        },

        _ => Pipe {
            x: 0,
            y: random_num(height as u64) as i16,
            dx: 1,
            dy: 0,
            color,
        },
    }
}

fn pipes(speed: u64) {
    let colors: [&str; 7] = [
        B_RED,
        B_GREEN,
        B_YELLOW,
        B_BLUE,
        B_MAGENTA,
        B_CYAN,
        B_WHITE,
    ];

    let mut pipes: Vec<Pipe> = Vec::new();
    let mut size: (i16, i16) = terminal_size();

    let mut grid: Vec<Vec<char>> =
        vec![vec![' '; size.0 as usize]; size.1 as usize];

    print!("\x1b[2J\x1b[H\x1b[?25l");
    io::stdout().flush().unwrap();

    let delay = Duration::from_millis(45 / speed);

    loop {
        let new_size = terminal_size();

        if new_size != size {
            size = new_size;

            grid = vec![vec![' '; size.0 as usize]; size.1 as usize];

            print!("\x1b[2J\x1b[H");

            pipes.retain(|pipe| {
                pipe.x >= 0 &&
                pipe.x < size.0 &&
                pipe.y >= 0 &&
                pipe.y < size.1
            });
        }

        let (width, height) = size;

        if random_num(5) == 0 || pipes.is_empty() {
            let color = colors[random_num(colors.len() as u64) as usize];
            pipes.push(random_pipe(width, height, color));
        }

        for pipe in &mut pipes {
            if pipe.x < 0 ||
               pipe.x >= width ||
               pipe.y < 0 ||
               pipe.y >= height {
                continue;
            }

            let x = pipe.x;
            let y = pipe.y;

            let character = if pipe.dx != 0 {
                '-'
            } else {
                '|'
            };

            let current = grid[y as usize][x as usize];

            if current == ' ' {
                grid[y as usize][x as usize] = character;

                print!(
                    "\x1b[{};{}H{}{}{}",
                    y + 1,
                    x + 1,
                    pipe.color,
                    character,
                    RESET
                );
            }

            pipe.x += pipe.dx;
            pipe.y += pipe.dy;

            if random_num(7) == 0 {
                if pipe.x >= 0 &&
                   pipe.x < width &&
                   pipe.y >= 0 &&
                   pipe.y < height {

                    let turn_x = pipe.x;
                    let turn_y = pipe.y;

                    grid[turn_y as usize][turn_x as usize] = '+';

                    print!(
                        "\x1b[{};{}H{}+{}",
                        turn_y + 1,
                        turn_x + 1,
                        pipe.color,
                        RESET
                    );
                }

                if pipe.dx != 0 {
                    pipe.dx = 0;
                    pipe.dy = if random_num(2) == 0 { 1 } else { -1 };
                } else {
                    pipe.dy = 0;
                    pipe.dx = if random_num(2) == 0 { 1 } else { -1 };
                }
            }
        }

        pipes.retain(|pipe| {
            pipe.x >= 0 &&
            pipe.x < width &&
            pipe.y >= 0 &&
            pipe.y < height
        });

        io::stdout().flush().unwrap();
        thread::sleep(delay);
    }
}

fn invalid_animation_err() {
    println!(
        "[ {B_RED}ERROR!{RESET} ] Invalid Animation Please Run With Valid Arguments"
    );
}

fn insufficient_args_err() {
    println!(
        "[ {B_RED}ERROR!{RESET} ] Insufficient Arguments Please Enter The Animation ID or Name"
    );
}

fn invalid_speed_err() {
    println!(
        "[ {B_RED}ERROR!{RESET} ] Speed Must Be Between 1x And 5x"
    );
}

fn load_animation(args: &[String]) {
    if args[0] == "0001" || args[0].to_lowercase() == "pipes" {
        let mut speed = 1;

        if args.len() >= 2 {
            if args[1] != "--speed" {
                invalid_speed_err();
                return;
            }

            if args.len() < 3 {
                invalid_speed_err();
                return;
            }

            match args[2].parse::<u64>() {
                Ok(value) if (1..=5).contains(&value) => {
                    speed = value;
                }

                _ => {
                    invalid_speed_err();
                    return;
                }
            }
        }

        pipes(speed);
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