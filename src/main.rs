use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use arboard::Clipboard;
use crossterm::event::KeyEvent;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App state
    let mut text = String::new();

    let mut last_pressed_key: Option<KeyEvent> = None;

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|f| {
            let area = f.area();
            let paragraph = Paragraph::new(text.as_str())
                .block(Block::default().title("Clipboard Viewer").borders(Borders::ALL))
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);

            // draw the last pressed key
            if let Some(key) = last_pressed_key {
                let key_info = format!("Last pressed key: {:?}", key);
                let key_paragraph = Paragraph::new(key_info)
                    .block(Block::default().title("Last Key").borders(Borders::ALL))
                    .wrap(Wrap { trim: false });
                f.render_widget(key_paragraph, area.inner(Margin { vertical: 1, horizontal: 1 }));
            }
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            last_pressed_key = Some(key);

            if key.code == KeyCode::Char('v') && key.modifiers.contains(KeyModifiers::CONTROL) {
                let mut clipboard = Clipboard::new()?;
                if let Ok(clip_text) = clipboard.get_text() {
                    text = clip_text;
                }
            } else if key.code == KeyCode::Enter {
                let re = Regex::new(r"\s+").unwrap();
                text = re.replace_all(&text, " ").to_string();
            } else if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                break;
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}