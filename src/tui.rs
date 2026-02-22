use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use std::io;

use crate::error::Result;
use crate::password;
use crate::vault::Vault;

enum AppState {
    List,
    Detail(usize),
    DeleteConfirm(usize),
    Add { website: String, username: String, focused: usize },
    Generate(String),
    Quit,
}

pub fn run(vault: &mut Vault) -> Result<bool> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::List;
    let mut list_state = TableState::default();
    if !vault.entries.is_empty() {
        list_state.select(Some(0));
    }
    let mut modified = false;
    let mut reveal_password = false;

    loop {
        terminal.draw(|f| {
            match &state {
                AppState::List => draw_list(f, vault, &mut list_state),
                AppState::Detail(idx) => draw_detail(f, vault, *idx, reveal_password),
                AppState::DeleteConfirm(idx) => {
                    draw_list(f, vault, &mut list_state);
                    draw_delete_confirm(f, vault, *idx);
                }
                AppState::Add { website, username, focused } => {
                    draw_add(f, website, username, *focused)
                }
                AppState::Generate(pw) => draw_generate(f, pw),
                AppState::Quit => {}
            }
        })?;

        if matches!(state, AppState::Quit) {
            break;
        }

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match &state {
                    AppState::List => match key.code {
                        KeyCode::Char('q') => state = AppState::Quit,
                        KeyCode::Down => {
                            if !vault.entries.is_empty() {
                                let i = list_state.selected().unwrap_or(0);
                                let next = (i + 1).min(vault.entries.len() - 1);
                                list_state.select(Some(next));
                            }
                        }
                        KeyCode::Up => {
                            if !vault.entries.is_empty() {
                                let i = list_state.selected().unwrap_or(0);
                                let prev = i.saturating_sub(1);
                                list_state.select(Some(prev));
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(idx) = list_state.selected() {
                                if idx < vault.entries.len() {
                                    reveal_password = false;
                                    state = AppState::Detail(idx);
                                }
                            }
                        }
                        KeyCode::Char('d') => {
                            if let Some(idx) = list_state.selected() {
                                if idx < vault.entries.len() {
                                    state = AppState::DeleteConfirm(idx);
                                }
                            }
                        }
                        KeyCode::Char('a') => {
                            state = AppState::Add {
                                website: String::new(),
                                username: String::new(),
                                focused: 0,
                            };
                        }
                        KeyCode::Char('g') => {
                            let pw = password::generate_password(20, true, true, true)
                                .unwrap_or_else(|_| "error".to_string());
                            state = AppState::Generate(pw);
                        }
                        _ => {}
                    },
                    AppState::Detail(_) => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            reveal_password = false;
                            state = AppState::List;
                        }
                        KeyCode::Char(' ') => {
                            reveal_password = !reveal_password;
                        }
                        _ => {}
                    },
                    AppState::DeleteConfirm(idx) => {
                        let idx = *idx;
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                vault.entries.remove(idx);
                                modified = true;
                                let new_selected = if vault.entries.is_empty() {
                                    None
                                } else {
                                    Some(idx.saturating_sub(if idx >= vault.entries.len() { 1 } else { 0 }))
                                };
                                list_state.select(new_selected);
                                state = AppState::List;
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                state = AppState::List;
                            }
                            _ => {}
                        }
                    }
                    AppState::Add { website, username, focused } => {
                        let mut w = website.clone();
                        let mut u = username.clone();
                        let f = *focused;
                        match key.code {
                            KeyCode::Char(c) => {
                                if f == 0 { w.push(c); } else { u.push(c); }
                                state = AppState::Add { website: w, username: u, focused: f };
                            }
                            KeyCode::Backspace => {
                                if f == 0 { w.pop(); } else { u.pop(); }
                                state = AppState::Add { website: w, username: u, focused: f };
                            }
                            KeyCode::Tab | KeyCode::Down | KeyCode::Up => {
                                state = AppState::Add { website: w, username: u, focused: 1 - f };
                            }
                            KeyCode::Enter => {
                                if f == 0 {
                                    state = AppState::Add { website: w, username: u, focused: 1 };
                                } else if !w.is_empty() && !u.is_empty() {
                                    let pw = password::generate_password(20, true, true, true)
                                        .unwrap_or_else(|_| "generated".to_string());
                                    vault.add_entry(w, u, pw);
                                    modified = true;
                                    list_state.select(Some(vault.entries.len() - 1));
                                    state = AppState::List;
                                }
                            }
                            KeyCode::Esc => {
                                state = AppState::List;
                            }
                            _ => {}
                        }
                    }
                    AppState::Generate(_) => match key.code {
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            let new_pw = password::generate_password(20, true, true, true)
                                .unwrap_or_else(|_| "error".to_string());
                            state = AppState::Generate(new_pw);
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            state = AppState::List;
                        }
                        _ => {}
                    },
                    AppState::Quit => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(modified)
}

fn styled_block() -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
}

fn draw_list(frame: &mut Frame, vault: &Vault, list_state: &mut TableState) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Header bar
    let entry_count = vault.entries.len();
    let count_text = format!(
        "{} {}  ",
        entry_count,
        if entry_count == 1 { "entry" } else { "entries" }
    );
    let header_block = styled_block();
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);

    let count_width = count_text.len() as u16;
    let header_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(count_width)])
        .split(header_inner);

    frame.render_widget(
        Paragraph::new(Span::styled(
            " * RUSTLOCK",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        header_cols[0],
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            count_text,
            Style::default().fg(Color::DarkGray),
        )),
        header_cols[1],
    );

    // Entry table
    let table_header = Row::new([
        Cell::from("Website").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Cell::from("Username").style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::UNDERLINED),
        ),
    ])
    .height(1);

    let rows: Vec<Row> = vault
        .entries
        .iter()
        .map(|e| {
            Row::new(vec![
                Cell::from(e.website.as_str())
                    .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from(e.username.as_str()).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Percentage(50), Constraint::Percentage(50)])
        .header(table_header)
        .block(styled_block())
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(table, chunks[1], list_state);

    // Help bar
    let help_spans = vec![
        Span::styled(" [↑↓] ", Style::default().fg(Color::Yellow)),
        Span::styled("Nav   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Enter] ", Style::default().fg(Color::Yellow)),
        Span::styled("View   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[a] ", Style::default().fg(Color::Yellow)),
        Span::styled("Add   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[g] ", Style::default().fg(Color::Yellow)),
        Span::styled("Generate   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[d] ", Style::default().fg(Color::Yellow)),
        Span::styled("Delete   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[q] ", Style::default().fg(Color::Yellow)),
        Span::styled("Quit", Style::default().fg(Color::DarkGray)),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(help_spans)).block(styled_block()),
        chunks[2],
    );
}

fn draw_detail(frame: &mut Frame, vault: &Vault, idx: usize, reveal_password: bool) {
    let area = frame.area();
    let entry = &vault.entries[idx];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(3)])
        .split(area);

    let password_display: &str = if reveal_password {
        &entry.password
    } else {
        "••••••••"
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "Website:  ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(&entry.website, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Username: ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(&entry.username, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Password: ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(password_display, Style::default().fg(Color::White)),
        ]),
    ];

    let detail_title = format!(" Entry: {} ", entry.website);
    let detail = Paragraph::new(lines).block(
        styled_block().title(Span::styled(
            detail_title,
            Style::default().fg(Color::White),
        )),
    );
    frame.render_widget(detail, chunks[0]);

    let reveal_label = if reveal_password { "Hide   " } else { "Reveal   " };
    let help_spans = vec![
        Span::styled(" [Space] ", Style::default().fg(Color::Yellow)),
        Span::styled(reveal_label, Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc] ", Style::default().fg(Color::Yellow)),
        Span::styled("Back", Style::default().fg(Color::DarkGray)),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(help_spans)).block(styled_block()),
        chunks[1],
    );
}

fn draw_add(frame: &mut Frame, website: &str, username: &str, focused: usize) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let outer = styled_block().title(Span::styled(
        " Add Entry ",
        Style::default().fg(Color::White),
    ));
    let inner = outer.inner(chunks[0]);
    frame.render_widget(outer, chunks[0]);

    // Horizontal padding inside the block
    let padded = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(2), Constraint::Min(0), Constraint::Length(2)])
        .split(inner)[1];

    let form = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top padding
            Constraint::Length(1), // "Website" label
            Constraint::Length(3), // website input
            Constraint::Length(1), // gap
            Constraint::Length(1), // "Username" label
            Constraint::Length(3), // username input
            Constraint::Length(1), // gap
            Constraint::Length(1), // note
            Constraint::Min(0),
        ])
        .split(padded);

    // Website label
    frame.render_widget(
        Paragraph::new(Span::styled(
            "Website",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        form[1],
    );

    // Website input field
    let website_cursor = if focused == 0 { "▌" } else { "" };
    let website_border = if focused == 0 {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!(" {}{}", website, website_cursor),
            Style::default().fg(Color::White),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(website_border),
        ),
        form[2],
    );

    // Username label
    frame.render_widget(
        Paragraph::new(Span::styled(
            "Username",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        form[4],
    );

    // Username input field
    let username_cursor = if focused == 1 { "▌" } else { "" };
    let username_border = if focused == 1 {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!(" {}{}", username, username_cursor),
            Style::default().fg(Color::White),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(username_border),
        ),
        form[5],
    );

    // Note
    frame.render_widget(
        Paragraph::new(Span::styled(
            "Password will be auto-generated (20 chars)",
            Style::default().fg(Color::DarkGray),
        )),
        form[7],
    );

    // Help bar
    let help_spans = vec![
        Span::styled(" [Tab] ", Style::default().fg(Color::Yellow)),
        Span::styled("Next field   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Enter] ", Style::default().fg(Color::Yellow)),
        Span::styled("Save   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc] ", Style::default().fg(Color::Yellow)),
        Span::styled("Cancel", Style::default().fg(Color::DarkGray)),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(help_spans)).block(styled_block()),
        chunks[1],
    );
}

fn draw_generate(frame: &mut Frame, password: &str) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let outer = styled_block().title(Span::styled(
        " Generate Password ",
        Style::default().fg(Color::White),
    ));
    let inner = outer.inner(chunks[0]);
    frame.render_widget(outer, chunks[0]);

    let padded = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(2), Constraint::Min(0), Constraint::Length(2)])
        .split(inner)[1];

    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // top padding
            Constraint::Length(1), // label
            Constraint::Length(3), // password box
            Constraint::Min(0),
        ])
        .split(padded);

    frame.render_widget(
        Paragraph::new(Span::styled(
            "Generated Password",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        content[1],
    );

    frame.render_widget(
        Paragraph::new(Span::styled(
            format!(" {} ", password),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        ),
        content[2],
    );

    let help_spans = vec![
        Span::styled(" [r] ", Style::default().fg(Color::Yellow)),
        Span::styled("Regenerate   ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc] ", Style::default().fg(Color::Yellow)),
        Span::styled("Back", Style::default().fg(Color::DarkGray)),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(help_spans)).block(styled_block()),
        chunks[1],
    );
}

fn draw_delete_confirm(frame: &mut Frame, vault: &Vault, idx: usize) {
    let area = frame.area();
    let entry = &vault.entries[idx];

    let popup_area = centered_rect(50, 7, area);

    let text = format!(
        "\n  Delete entry for '{}'?\n\n  Press y to confirm, n or Esc to cancel.",
        entry.website
    );
    let popup = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Confirm Delete ")
            .style(Style::default().fg(Color::Red)),
    );
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let width = area.width * percent_x / 100;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
