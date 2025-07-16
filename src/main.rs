use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph, Wrap}, DefaultTerminal};
use regex::Regex;

const ACCENT_COLOR: Color = Color::Red;

trait NiceKeyEvent {
    /// Returns a string representation of the key event for display purposes.
    fn to_nice_string(&self) -> String;
}

impl NiceKeyEvent for KeyEvent {
    fn to_nice_string(&self) -> String {
        if self.modifiers.is_empty() {
            format!("{}", self.code)
        } else {
            format!("{} + {}", self.modifiers.to_string(), self.code)
        }
    }
}

enum Action {
    PasteFromClipboard,
    RemoveExtraSpaces,
    CopyToClipboard,
    ClearText,
    QuickFix,
    Exit,
}

impl Action {
    /// Returns a string representation of the action for display purposes.
    fn as_str(&self) -> &str {
        match self {
            Action::PasteFromClipboard => "Pasted text from clipboard",
            Action::RemoveExtraSpaces => "Removed extra spaces",
            Action::CopyToClipboard => "Copied text to clipboard",
            Action::ClearText => "Cleared text",
            Action::QuickFix => "Quick fix applied. Your clipboard was updated.",
            Action::Exit => "Exiting application",
        }
    }
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
    fn paste_text_from_clipboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
            (KeyCode::F(1), _) => {
                self.last_action = Some(Action::QuickFix);
                self.paste_text_from_clipboard()?;
                self.remove_extra_spaces();
                self.copy_text_to_clipboard()
            },
            (KeyCode::F(2), _) => {
                self.last_action = Some(Action::PasteFromClipboard);
                self.paste_text_from_clipboard()
            },
            (KeyCode::F(3), _) => {
                self.last_action = Some(Action::RemoveExtraSpaces);
                self.remove_extra_spaces();
                Ok(())
            },
            (KeyCode::F(4), _) => {
                self.last_action = Some(Action::CopyToClipboard);
                self.copy_text_to_clipboard()
            },
            (KeyCode::F(5), _) => {
                self.last_action = Some(Action::ClearText);
                self.clear_text();
                Ok(())
            },
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.last_action = Some(Action::Exit);
                self.request_exit();
                Ok(())
            },
            _ => Ok(())
        };

        // store the last error message if it occurred
        if let Err(ref e) = result {
            let date = chrono::Local::now();
            let pretty_date = date.format("%Y-%m-%d %H:%M:%S").to_string();
            self.last_error = Some(format!("{}: {}", pretty_date, e.to_string()));
        }

        result
    }

    fn request_exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [instructions, textbox, audit_log, last_error] = Layout::vertical([
            Constraint::Length(3), // instructions
            Constraint::Min(5),    // text box
            Constraint::Length(3), // last action
            Constraint::Length(3), // error (optional)
        ]).vertical_margin(0).horizontal_margin(1).areas(area);

        let [last_key, last_action] = Layout::horizontal([
            Constraint::Length(20),
            Constraint::Fill(10),
        ]).areas(audit_log);

        // draw the instructions
        let text = "<F2> paste | <F3> remove space | <F4> copy to clipboard | <F5> clear text | <CTRL+c> exit";
        Paragraph::new(text)
            .block(Block::bordered()
                .title("Keybinds").title_style(Style::new().fg(ACCENT_COLOR)))
            .render(instructions, buf);

        // draw the text from the clipboard
        Paragraph::new(self.text.as_str())
            .block(Block::bordered()
                .title("Text Box (press F2 to paste)").title_style(Style::new().fg(Color::Red)))
            .wrap(Wrap { trim: false }).render(textbox, buf);

        // draw the last pressed key
        Block::bordered()
            .title("Last Key")
            .title_style(Style::new().fg(ACCENT_COLOR))
            .render(last_key, buf);

        if let Some(key) = self.last_pressed_key {
            Paragraph::new(key.to_nice_string())
                .wrap(Wrap { trim: false })
                .render(last_key.inner(Margin::new(1,1)), buf);
        }

        // draw the last action taken
        Block::bordered()
            .title("Last Action")
            .title_alignment(Alignment::Right)
            .title_style(Style::new().fg(ACCENT_COLOR))
            .render(last_action, buf);

        if let Some(action) = &self.last_action {
            let action_info = action.as_str();

            Paragraph::new(action_info)
                .wrap(Wrap { trim: false })
                .render(last_action.inner(Margin::new(1, 1)), buf);
        }

        // draw the last error message if it exists
        Block::bordered()
            .title("Last Error")
            .title_style(Style::new().fg(ACCENT_COLOR))
            .render(last_error, buf);
        if let Some(ref error) = self.last_error {
            Paragraph::new(error.as_str())
                .wrap(Wrap { trim: false })
                .render(last_error.inner(Margin::new(1, 1)), buf);
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