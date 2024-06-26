use std::{env, io::stdout};

use app::App;
use crossterm::{
    cursor::SetCursorStyle,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod config;
mod email;
mod widget;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    for arg in args {
        if arg == "--version" || arg == "-V" || arg == "-v" {
            println!(
                "tinbox v{}",
                option_env!("CARGO_PKG_VERSION").unwrap_or("UNKNOWN")
            );
            return Ok(());
        }
    }
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(SetCursorStyle::SteadyBar)?;
    stdout().execute(EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    app.run_app(&mut terminal).await?;

    disable_raw_mode()?;
    stdout().execute(DisableMouseCapture)?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
