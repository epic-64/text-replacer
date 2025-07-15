use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph, Wrap}, DefaultTerminal};
use regex::Regex;

enum Action {
    CopyFromClipboard,
    RemoveExtraSpaces,
    CopyToClipboard,
    ClearText,
    Exit,
}

#[derive(Default)]
struct App {
    exit: bool,
    clipboard: Option<Clipboard>,
    pub text: String,
    pub last_pressed_key: Option<KeyEvent>,
    pub last_error: Option<String>,
    pub last_action: Option<Action>,
}

// The basic application structure. Does not change. Can be copy-pasted into any application.
impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
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
    fn handle_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.on_key_pressed(key_event)
            }
            _ => Ok(()),
        }
    }
}

// The user logic for the application.
impl App {
    fn copy_text_from_clipboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut clipboard = Clipboard::new()?;
        if let Ok(clip_text) = clipboard.get_text() {
            self.text = clip_text;
        }
        Ok(())
    }

    fn copy_text_to_clipboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(clipboard) = &mut self.clipboard {
            clipboard.set_text(self.text.clone())?;
        } else {
            return Err("Clipboard unavailable".into());
        }
        Ok(())
    }

    fn remove_extra_spaces(&mut self) {
        let re = Regex::new(r"\s+").unwrap();
        self.text = re.replace_all(&self.text, " ").to_string();
    }

    fn clear_text(&mut self) {
        self.text.clear();
    }

    fn on_key_pressed(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.last_pressed_key = Some(key_event);

        let result = match (key_event.code, key_event.modifiers) {
            (KeyCode::F(2), _) => {
                self.last_action = Some(Action::CopyFromClipboard);
                self.copy_text_from_clipboard()
            },
            (KeyCode::F(3), _) => {
                self.last_action = Some(Action::RemoveExtraSpaces);
                Ok(self.remove_extra_spaces())
            },
            (KeyCode::F(4), _) => {
                self.last_action = Some(Action::CopyToClipboard);
                self.copy_text_to_clipboard()
            },
            (KeyCode::F(5), _) => {
                self.last_action = Some(Action::ClearText);
                Ok(self.clear_text())
            },
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.last_action = Some(Action::Exit);
                Ok(self.request_exit())
            },
            _ => Ok(())
        };

        // store the last error message if it occurred
        if let Err(ref e) = result {
            self.last_error = Some(e.to_string());
        }

        result
    }

    fn request_exit(&mut self) {
        self.last_error = Some("Exiting...".to_string());
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [instructions, textbox, last_key, last_error] = Layout::vertical([
            Constraint::Length(3), // instructions
            Constraint::Min(5),    // text box
            Constraint::Length(3), // last key
            Constraint::Length(3), // error (optional)
        ]).margin(1).areas(area);

        // draw the instructions
        let text = "<F2> paste | <F3> remove extra spaces | <F4> copy to clipboard | <F5> clear text | <CTRL+c> exit";
        Paragraph::new(text).render(instructions, buf);

        // draw the text from the clipboard
        Paragraph::new(self.text.as_str()).wrap(Wrap { trim: false }).render(textbox, buf);

        // draw the last pressed key
        if let Some(key) = self.last_pressed_key {
            let key_info = format!("Last pressed key: {:?}", key);

            Paragraph::new(key_info)
                .block(Block::default().title("Last Key").borders(Borders::ALL))
                .wrap(Wrap { trim: false })
                .render(last_key, buf);
        }

        // draw the last error message if it exists
        if let Some(ref error) = self.last_error {
            Paragraph::new(error.as_str())
                .block(Block::default().title("Last Error").borders(Borders::ALL))
                .wrap(Wrap { trim: false })
                .render(last_error, buf);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let mut app = App::default();
    app.clipboard = Clipboard::new().ok();
    app.run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}