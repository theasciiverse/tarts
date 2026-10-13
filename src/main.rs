#![cfg(not(test))]
use crossterm::{self, cursor, execute, terminal};
use std::{io, process};

mod analyze;
mod blank;
mod boids;
mod buffer;
mod butterfly;
mod check;
mod collision;
mod common;
mod config;
mod constellation;
mod crab;
mod cube;
mod donut;
mod error;
mod fire;
mod life;
mod maze;
mod pipes;
mod plasma;
mod pocketworld;
mod rain;

use crate::config::Config;

const VALID_SAVERS: &[&str] = &[
    "matrix",
    "life",
    "maze",
    "boids",
    "blank",
    "cube",
    "crab",
    "donut",
    "pipes",
    "plasma",
    "fire",
    "constellation",
    "butterfly",
    "pocketworld",
    "tour",
];

/// The effects `tarts tour` shows in turn, and how long each one runs
const TOUR: &[&str] = &[
    "matrix",
    "donut",
    "boids",
    "plasma",
    "fire",
    "pipes",
    "life",
    "cube",
    "constellation",
    "crab",
    "butterfly",
    "maze",
];
const TOUR_SECONDS: u64 = 20;

#[derive(Debug)]
struct AppArgs {
    screen_saver: String,
    check: bool,
    analyze: bool,
    effect: Option<String>,
    frames: Option<usize>,
    seed: Option<u32>,
    view: Option<String>,
    dump: bool,
    size: Option<(u16, u16)>,
}

/// Guard to drop out alternate screen in case of errors
struct TerminalGuard {
    stdout: io::Stdout,
}

impl TerminalGuard {
    fn new() -> Result<Self, io::Error> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )?;

        Ok(Self { stdout })
    }

    // Get mutable access to the stdout
    fn get_stdout(&mut self) -> &mut io::Stdout {
        &mut self.stdout
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Ignore errors during drop - we're doing best effort cleanup
        let _ = execute!(
            self.stdout,
            cursor::Show,
            terminal::Clear(terminal::ClearType::All),
            terminal::LeaveAlternateScreen,
        );
        let _ = terminal::disable_raw_mode();
    }
}

fn main() -> Result<(), error::TartsError> {
    env_logger::init();

    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing args: {}", e);
            process::exit(1);
        }
    };

    if args.analyze {
        let name = args.screen_saver.as_str();
        if !VALID_SAVERS.contains(&name) {
            eprintln!("Unknown screen saver: {}", name);
            print_help();
            process::exit(1);
        }
        let frames = args.frames.unwrap_or(1000);
        let seed = args.seed.unwrap_or(analyze::DEFAULT_SEED);
        let (width, height) = args
            .size
            .unwrap_or((analyze::DEFAULT_WIDTH, analyze::DEFAULT_HEIGHT));
        let opts = analyze::AnalysisOptions {
            frames,
            seed,
            view: args.view.clone(),
            dump: args.dump,
            width,
            height,
        };
        return match analyze::analyze(name, &opts) {
            Ok(report) => {
                println!("{}", report);
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        };
    }

    if args.check {
        let effect = args.effect.unwrap_or_else(|| "matrix".to_string());
        let frames = args.frames.unwrap_or(1);
        return check::run_test_for_effect(&effect, frames);
    }

    // Check if valid before entering alternate screen
    if !VALID_SAVERS.contains(&args.screen_saver.as_str()) {
        println!("Unknown screen saver: {}", args.screen_saver);
        print_help();
        return Ok(());
    }

    let (config, config_status) = Config::load()?;

    let fps = {
        let mut guard = TerminalGuard::new()?;
        // ptys/CI/panes can report 0 rows/cols; clamp so effects never see a
        // degenerate size (issue #94)
        let (width, height) = {
            let (w, h) = terminal::size()?;
            (w.max(1), h.max(1))
        };

        if args.screen_saver == "tour" {
            tour(guard.get_stdout(), &config)?;
            None
        } else {
            let mut effect =
                make_effect(&args.screen_saver, &config, (width, height))
                    .expect("checked against VALID_SAVERS");
            Some(common::run_loop(guard.get_stdout(), effect.as_mut(), None)?)
        }
    };

    println!("{}", config_status);
    if let Some(fps) = fps {
        println!("Frames per second: {:.1}", fps);
    }
    Ok(())
}

/// The effect called `name`, sized to the terminal
fn make_effect(
    name: &str,
    config: &Config,
    (width, height): (u16, u16),
) -> Option<Box<dyn common::TerminalEffect>> {
    let effect: Box<dyn common::TerminalEffect> = match name {
        "matrix" => {
            let options = config.get_matrix_options((width, height));
            let digital_rain =
                rain::digital_rain::DigitalRain::new(options, (width, height));
            Box::new(digital_rain)
        }
        "life" => {
            let options = config.get_life_options((width, height));
            let conway_life = life::ConwayLife::new(options, (width, height));
            Box::new(conway_life)
        }
        "maze" => {
            let options = config.get_maze_options((width, height));
            let maze = maze::Maze::new(options, (width, height));
            Box::new(maze)
        }
        "boids" => {
            let options = config.get_boids_options((width, height));
            let boids = boids::Boids::new(options);
            Box::new(boids)
        }
        "blank" => {
            let options = config.get_blank_options();
            let blank = blank::Blank::new(options, (width, height));
            Box::new(blank)
        }
        "cube" => {
            let options = config.get_cube_options();
            let cube = cube::Cube::new(options, (width, height));
            Box::new(cube)
        }
        "crab" => {
            let options = config.get_crab_options((width, height));
            let crab = crab::Crab::new(options, (width, height));
            Box::new(crab)
        }
        "donut" => {
            let options = config.get_donut_options((width, height));
            let donut = donut::Donut::new(options, (width, height));
            Box::new(donut)
        }
        "pipes" => {
            let options = config.get_pipes_options();
            let pipes = pipes::Pipes::new(options, (width, height));
            Box::new(pipes)
        }
        "plasma" => {
            let options = config.get_plasma_options();
            let plasma = plasma::Plasma::new(options, (width, height));
            Box::new(plasma)
        }
        "fire" => {
            let options = config.get_fire_options();
            let fire = fire::Fire::new(options, (width, height));
            Box::new(fire)
        }
        "constellation" => {
            let options = config.get_constellation_options();
            let constellation =
                constellation::Constellation::new(options, (width, height));
            Box::new(constellation)
        }
        "butterfly" => {
            let options = config.get_butterfly_options((width, height));
            let butterfly = butterfly::Butterfly::new(options, (width, height));
            Box::new(butterfly)
        }
        "pocketworld" => {
            let options = config.get_pocketworld_options();
            let pocketworld =
                pocketworld::PocketWorld::new(options, (width, height));
            Box::new(pocketworld)
        }
        _ => return None,
    };
    Some(effect)
}

/// Every effect in turn, TOUR_SECONDS each, round and round until quit;
/// n, Space or Right skip ahead, p or Left go back
fn tour(stdout: &mut io::Stdout, config: &Config) -> Result<(), error::TartsError> {
    let mut at = 0;
    loop {
        // clamp a degenerate size, as for a single effect (issue #94)
        let size = {
            let (w, h) = terminal::size()?;
            (w.max(1), h.max(1))
        };
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        let mut effect =
            make_effect(TOUR[at], config, size).expect("TOUR holds known effects");
        let length = std::time::Duration::from_secs(TOUR_SECONDS);
        match common::run_timed(stdout, effect.as_mut(), length)? {
            common::TourStep::Quit => return Ok(()),
            common::TourStep::Next => at = (at + 1) % TOUR.len(),
            common::TourStep::Previous => at = (at + TOUR.len() - 1) % TOUR.len(),
        }
    }
}

fn parse_args() -> Result<AppArgs, String> {
    let mut args = std::env::args().skip(1);
    let mut screen_saver = "matrix".to_string();
    let mut check = false;
    let mut analyze = false;
    let mut effect = None;
    let mut frames = None;
    let mut seed = None;
    let mut view = None;
    let mut dump = false;
    let mut size = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-v" => {
                print_version();
                std::process::exit(0);
            }
            "--check" => {
                check = true;
            }
            "--analyze" => {
                analyze = true;
            }
            "--seed" => {
                if let Some(seed_str) = args.next() {
                    seed = seed_str.parse().ok();
                }
            }
            "--view" => {
                view = args.next();
            }
            "--dump" => {
                dump = true;
            }
            "--size" => {
                if let Some(size_str) = args.next() {
                    size = Some(parse_size(&size_str)?);
                }
            }
            "--print-config" => {
                if let Err(e) = Config::print_default_config() {
                    eprintln!("Failed to print config: {}", e);
                    std::process::exit(1);
                }
                std::process::exit(0);
            }
            "--effect" => {
                effect = args.next();
            }
            "--frames" => {
                if let Some(frame_str) = args.next() {
                    frames = frame_str.parse().ok();
                }
            }
            arg if !arg.starts_with('-') => {
                if check {
                    effect = Some(arg.to_string());
                } else {
                    screen_saver = arg.to_string();
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", arg));
            }
        }
    }

    Ok(AppArgs {
        screen_saver,
        check,
        analyze,
        effect,
        frames,
        seed,
        view,
        dump,
        size,
    })
}

/// Parse a `WxH` canvas spec (e.g. `160x50`).
fn parse_size(s: &str) -> Result<(u16, u16), String> {
    let (w, h) = s.split_once('x').ok_or_else(|| {
        format!("invalid size '{}' (expected WxH, e.g. 160x50)", s)
    })?;
    let width: u16 = w
        .parse()
        .map_err(|_| format!("invalid width in '{}' (expected WxH)", s))?;
    let height: u16 = h
        .parse()
        .map_err(|_| format!("invalid height in '{}' (expected WxH)", s))?;
    Ok((width, height))
}

fn print_help() {
    println!("tarts - Terminal screensavers");
    println!();
    println!("USAGE:");
    println!("    tarts [EFFECT] [OPTIONS]");
    println!();
    println!("EFFECTS:");
    println!("    matrix      Matrix digital rain");
    println!("    life        Conway's Game of Life");
    println!("    maze        Maze generation");
    println!("    boids       Boids flocking simulation");
    println!("    cube        3D cube rotation");
    println!("    crab        ASCII crab animation");
    println!("    donut       3D donut rotation");
    println!("    pipes       Pipe maze animation");
    println!("    plasma      Plasma effect");
    println!("    fire        Fire simulation");
    println!("    pocketworld Living pocket world: rivers, rain, plates, seasons");
    println!("    constellation  Drifting stars and dotted connections");
    println!("    butterfly   Butterflies floating around");
    println!("    blank       Blank screen");
    println!("    tour        Every effect in turn (n next, p previous)");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help              Show help");
    println!("    -v, --version           Show version");
    println!("        --check             Run test mode");
    println!("        --effect <EFFECT>    Effect to test (with --check)");
    println!("        --frames <NUM>       Number of frames to run (with --check)");
    println!(
        "        --analyze <EFFECT>   Headless render analysis: glyph census + change rate"
    );
    println!("        --seed <NUM>         Seed for --analyze (default 7)");
    println!(
        "        --view <NAME>        View for --analyze (pocketworld: biome/plates/height/hydro/temp/wind/humidity/aquifer)"
    );
    println!("        --print-config       Print default config as TOML to stdout");
    println!();
    println!("CONFIG:");
    println!("    Config file (optional): ~/.config/tarts.toml");
    println!(
        "    Generate one with:      tarts --print-config > ~/.config/tarts.toml"
    );
    println!();
    println!("EXAMPLES:");
    println!("    tarts matrix            Run Matrix effect");
    println!("    tarts --check            Test with default effect");
    println!("    tarts --check life       Test Life effect");
    println!("    tarts --check --frames 100 life");
    println!("    tarts --analyze pocketworld --view height --frames 3000");
    println!("    tarts --analyze pocketworld --view biome --size 160x50 --dump");
    println!("    tarts --print-config > ~/.config/tarts.toml");
    println!("    tarts --version          Show version");
}

fn print_version() {
    println!("tarts {}", env!("CARGO_PKG_VERSION"));
}
