use crossterm::{
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{
    error::Error,
    io::{self, Write},
};

struct App {
    input_mode: bool,
    url: String,
    message: String,
}

impl App {
    fn new() -> App {
        App {
            input_mode: false,
            url: String::new(),
            message: String::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableBracketedPaste)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    if app.input_mode {
                        match key.code {
                            KeyCode::Enter => {
                                let url = app.url.clone();
                                let fetch_result = reqwest::blocking::get(&url);
                                match fetch_result {
                                    Ok(mut resp) => {
                                        if resp.status().is_success() {
                                            match resp.text() {
                                                Ok(text) => {
                                                    if let Ok(mut file) = std::fs::File::create("fetched.html") {
                                                        if file.write_all(text.as_bytes()).is_ok() {
                                                            app.message = "Saved to fetched.html".to_string();
                                                        } else {
                                                            app.message = "Error writing to file".to_string();
                                                        }
                                                    } else {
                                                        app.message = "Error creating file".to_string();
                                                    }
                                                }
                                                Err(e) => {
                                                    app.message = format!("Error reading response: {}", e);
                                                }
                                            }
                                        } else {
                                            app.message = format!("HTTP error: {}", resp.status());
                                        }
                                    }
                                    Err(e) => {
                                        app.message = format!("Error fetching URL: {}", e);
                                    }
                                }
                                app.input_mode = false;
                            }
                            KeyCode::Char(c) => {
                                app.url.push(c);
                            }
                            KeyCode::Backspace => {
                                app.url.pop();
                            }
                            KeyCode::Esc => {
                                app.input_mode = false;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('u') => {
                                app.input_mode = true;
                                app.url.clear();
                                app.message.clear();
                            }
                            _ => {}
                        }
                    }
                }
            }
            Event::Paste(data) => {
                if app.input_mode {
                    app.url.push_str(&data);
                }
            }
            _ => {}
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(f.area());

    let keybinds = Paragraph::new("u: Enter URL\nq: Quit")
        .block(Block::default().title("Keybinds").borders(Borders::ALL));
    f.render_widget(keybinds, chunks[0]);

    if app.input_mode {
        let input = Paragraph::new(app.url.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().title("URL Input").borders(Borders::ALL));
        f.render_widget(input, chunks[1]);
    } else {
        let message = Paragraph::new(app.message.as_str())
            .block(Block::default().title("Message").borders(Borders::ALL));
        f.render_widget(message, chunks[1]);
    }
}
