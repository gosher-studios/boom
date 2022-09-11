use std::{io, process};
use tui::{
  backend::CrosstermBackend,
  Terminal,
  layout::{Layout, Alignment, Constraint, Direction},
  widgets::{Block, Borders, Paragraph},
};
use crate::Result;
use crossterm::terminal::{
  enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::execute;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use figlet_rs::FIGfont;
use chrono::{Duration, Utc, Datelike};
pub fn menu() -> Result {
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

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
            Constraint::Min(0),
            Constraint::Percentage(50)
          ]
          .as_ref(),
        )
        .split(hchunks[0]);
      let shit = Paragraph::new(logo.clone());
      f.render_widget(shit, vchunks[0]);
      let fart = Paragraph::new("ok so liek there is a fucking amount of letters and you have to find a word with that amount of letters inside of it like wow no fucking wooooo")
        .block(Block::default().title("How To Play").borders(Borders::ALL));
      f.render_widget(fart, vchunks[1]);

     let changelog = Paragraph::new("we added shit")
        .block(Block::default().title("Changelog").borders(Borders::ALL));
      f.render_widget(changelog, vchunks[2]);
      let block = Block::default().title("").borders(Borders::ALL);
      f.render_widget(block, hchunks[1]);
    })?;

    if event::poll(Duration::zero().to_std()?)? {
      if let Event::Key(key) = event::read()? {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
          match key.code {
            KeyCode::Char('q') => {
              disable_raw_mode()?;
              execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
              process::exit(0);
            }
            _ => {}
          }
        }
      }
    }
  }
  // Ok(())
}
