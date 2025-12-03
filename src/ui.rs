use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;

#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

pub struct App {
    pub messages: Vec<Message>,
    pub input: String,
    pub should_quit: bool,
    pub is_processing: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            messages: vec![Message {
                role: MessageRole::System,
                content: "Welcome to Ada. Type /help for commands, or chat with GPT-4.".to_string(),
            }],
            input: String::new(),
            should_quit: false,
            is_processing: false,
        }
    }

    pub fn add_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(Message { role, content });
    }

    pub fn submit_input(&mut self) -> Option<String> {
        if self.input.is_empty() {
            return None;
        }

        let input = self.input.clone();
        self.input.clear();
        Some(input)
    }
}

pub struct UI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl UI {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.show_cursor()?;

        Ok(Self { terminal })
    }

    pub fn draw(&mut self, app: &App) -> Result<()> {
        self.terminal.draw(|f| {
            let size = f.area();

            // Split screen into messages (top) and input (bottom, fixed height)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(size.height.saturating_sub(3)),  // Messages
                    Constraint::Length(3),                               // Input box
                ])
                .split(size);

            render_messages(f, app, chunks[0]);
            render_input(f, app, chunks[1]);
        })?;

        // Position cursor in the input box
        let input_len = app.input.len() as u16;
        let input_area = self.terminal.size()?;
        execute!(
            self.terminal.backend_mut(),
            cursor::MoveTo(input_len + 1, input_area.height - 2)
        )?;

        Ok(())
    }

    pub fn handle_events(&self, app: &mut App) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            app.should_quit = true;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Enter => {
                            if let Some(input) = app.submit_input() {
                                app.add_message(MessageRole::User, input);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen
        );
        let _ = self.terminal.show_cursor();
    }
}

fn render_messages(f: &mut Frame, app: &App, area: Rect) {
    let mut text = String::new();

    for msg in &app.messages {
        match msg.role {
            MessageRole::User => {
                text.push_str("> ");
                text.push_str(&msg.content);
                text.push_str("\n\n");
            }
            MessageRole::Assistant => {
                text.push_str("⏺ ");
                text.push_str(&msg.content);
                text.push_str("\n\n");
            }
            MessageRole::System => {
                text.push_str(&msg.content);
                text.push_str("\n\n");
            }
        }
    }

    // Add working indicator if processing
    if app.is_processing {
        text.push_str("✢ Working… (esc to interrupt)\n");
    }

    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .scroll((0, 0));

    f.render_widget(paragraph, area);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let input_text = if app.is_processing {
        String::new()
    } else {
        app.input.clone()
    };

    let input = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input (Enter to send, Ctrl+C to quit) ")
                .style(Style::default().fg(Color::Cyan))
        )
        .wrap(Wrap { trim: false });

    f.render_widget(input, area);
}
