use crossterm::event::KeyCode;
use crossterm::style::{Attribute, Color, Stylize};
use crossterm::terminal::ClearType;
use crossterm::{ExecutableCommand, cursor, event, queue, style, terminal};

use std::io::{Stdout, Write};

use crate::decode::decode_string;
use crate::encode::encode_string;

/// Set a panic hook to restore terminal state on panic
/// This ensures that the terminal is not left in raw mode or alternate screen on panic
/// even if the panic occurs in a different thread
fn set_panic_hook() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal(&mut std::io::stdout()); // ignore any errors as we are already failing
        hook(panic_info);
        std::process::exit(1); // exit immediately after restoring terminal
    }));
}

/// Setup terminal in raw mode and enter alternate screen
/// Also sets a panic hook to restore terminal on panic
pub fn setup_terminal(stdout: &mut Stdout) -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    set_panic_hook();
    crossterm::queue!(
        stdout,
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0)
    )?;
    stdout.flush()?;
    Ok(())
}

/// Restore terminal to original state
/// Leave alternate screen and disable raw mode
pub fn restore_terminal(stdout: &mut Stdout) -> std::io::Result<()> {
    queue!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    stdout.flush()?;
    terminal::disable_raw_mode()?;
    Ok(())
}

pub fn run(stdout: &mut Stdout) -> std::io::Result<()> {
    // Title
    stdout.execute(style::PrintStyledContent(
        "Base64 Live Encoder/Decoder\r\n"
            .with(Color::Blue)
            .attribute(Attribute::Bold),
    ))?;

    let mut input = String::new();

    // Save cursor position so we can restore & redraw
    queue!(stdout, cursor::Hide, cursor::SavePosition)?;
    stdout.flush()?;

    loop {
        // Re-render
        queue!(
            stdout,
            cursor::RestorePosition,
            terminal::Clear(ClearType::FromCursorDown)
        )?;

        // Print prompt and input
        queue!(
            stdout,
            style::PrintStyledContent(
                "Input string: "
                    .with(Color::Cyan)
                    .attribute(Attribute::Bold),
            ),
            style::Print(&input),
            style::Print("⏎"),
            style::Print("\r\n"),
        )?;
        stdout.flush()?;

        // Print encoded string
        let encoded = encode_string(&input);
        queue!(
            stdout,
            style::PrintStyledContent(
                "Base64 Encoded: "
                    .with(Color::Green)
                    .attribute(Attribute::Bold),
            ),
            style::Print(&encoded.with(Color::Yellow)),
            style::Print(" \r\n"),
        )?;
        stdout.flush()?;

        // Print decoded string
        let decoded = match decode_string(&input) {
            Some(s) => s.with(Color::Yellow),
            None => "<invalid input>".to_string().with(Color::Red),
        };
        queue!(
            stdout,
            style::PrintStyledContent(
                "Base64 Decoded: "
                    .with(Color::Green)
                    .attribute(Attribute::Bold),
            ),
            style::Print(&decoded),
            style::Print(" \r\n"),
        )?;
        stdout.flush()?;

        // Wait for key event
        if let event::Event::Key(event::KeyEvent { code, kind, .. }) = event::read()? {
            match code {
                KeyCode::Char(c) if kind == event::KeyEventKind::Press => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Esc => {
                    // User cancelled input. Exit loop.
                    break;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
