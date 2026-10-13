use crate::buffer::Cell;
use crossterm::{
    QueueableCommand, cursor, event,
    style::{self, Stylize},
    terminal,
};
use std::{
    io::{BufWriter, Result, Write},
    time::Duration,
};

pub trait DefaultOptions {
    type Options;

    /// Calculate default options based on terminal dimensions
    #[allow(unused)]
    fn default_options(width: u16, height: u16) -> Self::Options;
}

pub trait TerminalEffect {
    /// get difference between frames, this is used to minimize screen updates
    fn get_diff(&mut self) -> Vec<(usize, usize, Cell)>;
    /// Update to next frame
    fn update(&mut self);
    // Update screen size option, each saver should implement it by itself
    fn update_size(&mut self, width: u16, height: u16);
    /// Reset effect, i think it's useful in case of size/options update
    fn reset(&mut self);
    /// Non-exit key press. Default: ignore. Effects that want display
    /// controls (e.g. terrain's layer cycling) override this.
    fn handle_key(&mut self, _key: event::KeyEvent) {}
    /// Renderable screens this effect exposes. Most effects have
    /// exactly one — their default screen.
    fn views(&self) -> Vec<&'static str> {
        vec!["default"]
    }
    /// Switch the active view to a name from `views`. Default: no-op —
    /// single-view effects have nothing to switch.
    fn set_view(&mut self, _name: &str) {}
}

pub fn process_input<TE>(effect: &mut TE) -> Result<bool>
where
    TE: TerminalEffect + ?Sized,
{
    while event::poll(Duration::from_millis(10))? {
        match event::read()? {
            event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('q') | event::KeyCode::Esc,
                ..
            })
            | event::Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('c'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            }) => return Ok(false),

            event::Event::Key(key) => effect.handle_key(key),

            event::Event::Resize(new_width, new_height) => {
                // clamp degenerate sizes (issue #94) before the effect resets
                effect.update_size(new_width.max(1), new_height.max(1));
                effect.reset();
            }

            _ => {}
        }
    }

    Ok(true)
}

pub fn run_loop<W, TE>(
    stdout: &mut W,
    effect: &mut TE,
    iterations: Option<usize>,
) -> Result<f64>
where
    W: Write,
    TE: TerminalEffect + ?Sized,
{
    let (width, height) = terminal::size()?;

    // #[cfg(test)]
    let mut iters: usize = 0;

    let mut is_running = true;
    let mut frames_per_second = 0.0;
    let target_frame_duration = Duration::from_secs_f64(1.0 / 60.0_f64);

    // wrap in buffer due to tests "run_loop_fps_gte_0" failing on CI/CD
    // NOTE: 12/Dec/2023 issue with tests of CI/CD still not resolved
    let mut buffered_stdout = BufWriter::new(stdout);

    // main loop
    while is_running {
        let started_at: std::time::SystemTime = std::time::SystemTime::now();

        // process_input
        is_running = process_input(effect)?;

        // draw diff
        let queue = effect.get_diff();
        for item in queue.iter() {
            let (x, y, cell) = item;
            debug_assert!(*x < width as usize && *y < height as usize);
            buffered_stdout.queue(cursor::MoveTo(*x as u16, *y as u16))?;
            buffered_stdout.queue(style::PrintStyledContent(
                cell.symbol.with(cell.color).attribute(cell.attr),
            ))?;
        }
        buffered_stdout.flush()?;
        effect.update();

        // stabilize fps if required
        let ended_at = std::time::SystemTime::now();
        let delta = ended_at
            .duration_since(started_at)
            .unwrap_or(std::time::Duration::from_secs(0));
        if delta < target_frame_duration {
            std::thread::sleep(target_frame_duration - delta);
        };

        // calculate actual frame rate
        let ended_at = std::time::SystemTime::now();
        let delta = ended_at
            .duration_since(started_at)
            .unwrap_or(std::time::Duration::from_secs(0));
        frames_per_second = (frames_per_second + (1.0 / delta.as_secs_f64())) / 2.0;

        // #[cfg(test)]
        if let Some(iterations) = iterations {
            iters += 1;
            if iters > iterations {
                is_running = false;
            }
        };
    }
    Ok(frames_per_second)
}

/// How a timed run in the tour ended
pub enum TourStep {
    Quit,
    Next,
    Previous,
}

/// Runs an effect for `length`, or until a key moves the tour on: n, Space
/// or Right for the next effect, p or Left for the previous one; q, Esc and
/// Ctrl+C quit as everywhere else.
pub fn run_timed<W, TE>(
    stdout: &mut W,
    effect: &mut TE,
    length: Duration,
) -> Result<TourStep>
where
    W: Write,
    TE: TerminalEffect + ?Sized,
{
    let target_frame_duration = Duration::from_secs_f64(1.0 / 60.0_f64);
    let mut buffered_stdout = BufWriter::new(stdout);
    let ends_at = std::time::Instant::now() + length;

    while std::time::Instant::now() < ends_at {
        let started_at = std::time::Instant::now();

        while event::poll(Duration::from_millis(10))? {
            match event::read()? {
                event::Event::Key(key)
                    if key.kind == event::KeyEventKind::Press =>
                {
                    match key.code {
                        event::KeyCode::Char('q') | event::KeyCode::Esc => {
                            return Ok(TourStep::Quit);
                        }
                        event::KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(event::KeyModifiers::CONTROL) =>
                        {
                            return Ok(TourStep::Quit);
                        }
                        event::KeyCode::Char('n')
                        | event::KeyCode::Char(' ')
                        | event::KeyCode::Right => return Ok(TourStep::Next),
                        event::KeyCode::Char('p') | event::KeyCode::Left => {
                            return Ok(TourStep::Previous);
                        }
                        _ => {}
                    }
                }
                event::Event::Resize(new_width, new_height) => {
                    buffered_stdout
                        .queue(terminal::Clear(terminal::ClearType::All))?;
                    // clamp degenerate sizes, as run_loop does (issue #94)
                    effect.update_size(new_width.max(1), new_height.max(1));
                    effect.reset();
                }
                _ => {}
            }
        }

        for (x, y, cell) in effect.get_diff().iter() {
            buffered_stdout.queue(cursor::MoveTo(*x as u16, *y as u16))?;
            buffered_stdout.queue(style::PrintStyledContent(
                cell.symbol.with(cell.color).attribute(cell.attr),
            ))?;
        }
        buffered_stdout.flush()?;
        effect.update();

        let spent = started_at.elapsed();
        if spent < target_frame_duration {
            std::thread::sleep(target_frame_duration - spent);
        }
    }
    Ok(TourStep::Next)
}
