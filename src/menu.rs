use std::{io, process};
use tui::{Terminal, backend::CrosstermBackend};
use tui::widgets::{Block, Borders, Paragraph, List, ListItem};
use tui::layout::{Layout, Direction, Constraint};
use tui::text::{Span, Spans};
use tui::style::{Style, Color};
use crossterm::terminal::{
  enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use figlet_rs::FIGfont;
use chrono::{Duration, Utc, Datelike};
use crate::Result;


pub fn menu() -> Result {
  let mut stdout = io::stdout();
  enable_raw_mode()?;
  crossterm::execute!(stdout, EnterAlternateScreen)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let mut selected = 0;
  let mut buf = String::new();
  let mut logo = FIGfont::from_content(include_str!("slant.flf"))?
    .convert("Boom Room")
    .unwrap()
    .to_string();
  logo.push_str(&format!(
    "v{} - © Gosher Studios {}",
    env!("CARGO_PKG_VERSION"),
    Utc::now().year()
  ));

  loop {
    terminal.draw(|f| {
      let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(f.size());
      let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
          [
            Constraint::Length(logo.lines().count().try_into().unwrap()),
            Constraint::Percentage(40),
            Constraint::Min(0),
          ]
          .as_ref(),
        )
        .split(hchunks[0]);
      f.render_widget(Paragraph::new(logo.clone()), vchunks[0]);
      f.render_widget(
        Paragraph::new("cum shit fart poop").block(Block::default().title("How To Play").borders(Borders::ALL)),
        vchunks[1],
      );
      f.render_widget(
        Paragraph::new("log").block(Block::default().title("Changelog").borders(Borders::ALL)),
        vchunks[2],
      );

      let option_item = |name: String, index: u8, prompt: String| {
        let mut spans = vec![Span::raw(name)];
        if selected == index {
          spans.push(Span::raw(" "));
          spans.push(Span::styled(
            if buf.is_empty() {
              prompt.clone()
            } else {
              buf.clone()
            },
            Style::default().fg(Color::DarkGray),
          ));
          spans.push(Span::raw(" <<"));
        }
        ListItem::new(Spans::from(spans))
      };
      f.render_widget(
        List::new(vec![
          option_item("Play".to_string(), 0, "Enter IP".to_string()),
          option_item("Host".to_string(), 1, "Enter port".to_string())
        ])
        .block(
          Block::default()
            .title("'▲▼' to select - 'ctrl+q' to exit")
            .borders(Borders::ALL),
        ),
        hchunks[1],
      );
    })?;

    if event::poll(Duration::zero().to_std()?)? {
      if let Event::Key(key) = event::read()? {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
          match key.code {
            KeyCode::Char('q') => {
              disable_raw_mode()?;
              crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
              process::exit(0);
            }
            _ => {}
          }
        } else {
          match key.code {
            KeyCode::Up => {
              buf.clear();
              selected = 0;
            }
            KeyCode::Down => {
              buf.clear();
              selected = 1;
            }
            KeyCode::Enter => {
              match selected {
                // todo
                0 => {} // i forgore
                1 => {} // goofy server mechanics remember lmao poopy stinky
                _ => {}
              }
            }
            KeyCode::Char(c) => {
              buf.push(c);
            }
            KeyCode::Backspace => {
              buf.pop();
            }
            _ => {}
          }
        }
      }
    }
  }
}
