use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph, Wrap}, DefaultTerminal};
use arboard::Clipboard;
use regex::Regex;

#[derive(Default)]
struct App {
    exit: bool,
    pub text: String,
    pub last_pressed_key: Option<KeyEvent>,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        let _ = match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => Ok(()),
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.last_pressed_key = Some(key_event);

        match (key_event.code, key_event.modifiers) {
            (KeyCode::F(2), _) => {
                // Paste from clipboard
                let mut clipboard = Clipboard::new()?;
                if let Ok(clip_text) = clipboard.get_text() {
                    self.text = clip_text;
                }
            }
            (KeyCode::F(3), _) => {
                // Copy current buffer to clipboard
                let mut clipboard = Clipboard::new()?;
                clipboard.set_text(self.text.clone())?;
            }
            (KeyCode::Enter, _) => {
                let re = Regex::new(r"\s+").unwrap();
                self.text = re.replace_all(&self.text, " ").to_string();
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.request_exit();
            }
            _ => {}
        }

        Ok(())
    }

    fn request_exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // draw the main block
        Block::default().title("Clipboard Viewer").borders(Borders::ALL).render(area, buf);

        // draw the instructions
        Paragraph::new("Press F2 to paste from clipboard, F3 to copy current buffer to clipboard, Enter to remove extra spaces.")
            .wrap(Wrap { trim: false })
            .render(area.inner(Margin { vertical: 1, horizontal: 1 }), buf);

        // draw the text from the clipboard
        let area = area;
        Paragraph::new(self.text.as_str())
            .block(Block::default().title("Clipboard Viewer").borders(Borders::ALL))
            .wrap(Wrap { trim: false })
            .render(area.inner(Margin { vertical: 1, horizontal: 1 }), buf);

        // draw the last pressed key
        if let Some(key) = self.last_pressed_key {
            let key_info = format!("Last pressed key: {:?}", key);

            Paragraph::new(key_info)
                .block(Block::default().title("Last Key").borders(Borders::ALL))
                .wrap(Wrap { trim: false })
                .render(area.inner(Margin { vertical: 1, horizontal: 1 }), buf);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    app.run(&mut terminal)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}