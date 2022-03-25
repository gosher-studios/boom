use std::{io, thread, process};
use std::time::Duration;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use tui::{Terminal, backend::CrosstermBackend};
use tui::widgets::{Block, Borders, List, ListItem};
use tui::layout::{Layout, Direction, Constraint};
use crossterm::terminal::{
  enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use chrono::prelude::*;

use crate::state::{State, StateChange, ClientPlayer};
use crate::Result;

pub struct Client {
  state: Arc<Mutex<State<ClientPlayer>>>,
  players_open: bool,
  chat_selected: bool,
  chat_buf: String,
}

impl Client {
  pub fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(State::new(String::new()))),
      players_open: false,
      chat_selected: false,
      chat_buf: String::new(),
    }
  }

  pub fn play(mut self, name: String) -> Result {
    let stream = TcpStream::connect("localhost:1234")?;
    bincode::serialize_into(&stream, &name)?;
    let mut state = self.state.lock().unwrap();
    *state = bincode::deserialize_from(&stream)?;
    state
      .chat
      .push(format!("Connected to {}", stream.peer_addr()?));
    drop(state);

    let state = self.state.clone();
    let s = stream.try_clone()?;
    thread::spawn(move || -> Result {
      let mut stdout = io::stdout();
      enable_raw_mode()?;
      crossterm::execute!(stdout, EnterAlternateScreen)?;
      let backend = CrosstermBackend::new(stdout);
      let mut terminal = Terminal::new(backend)?;

      loop {
        terminal.draw(|f| {
          let state = state.lock().unwrap();
          let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(f.size());
          let mut game: Vec<ListItem> = state
            .players
            .iter()
            .map(|(i, p)| {
              ListItem::new(format!(
                "{}{} - '{}'",
                if state.current_player == *i { "> " } else { "" },
                p.name,
                p.buf
              ))
            })
            .collect();
          let timer = Utc::now() - state.timer;
          game.insert(
            0,
            ListItem::new(format!(
              ">> {} << - {:02}:{:.2}",
              state.current_phrase,
              timer.num_seconds(),
              timer.num_milliseconds().to_string()
            )),
          );
          f.render_widget(
            List::new(game).block(
              Block::default()
                .title("Boom Room - 'ctrl+q' to exit")
                .borders(Borders::ALL),
            ),
            chunks[0],
          );

          let side = if self.players_open {
            let mut items: Vec<ListItem> = state
              .players
              .iter()
              .map(|(_, p)| ListItem::new(p.name.clone()))
              .collect();
            items.insert(
              0,
              ListItem::new(format!(
                "{}/{} players",
                state.players.len(),
                state.max_players
              )),
            );
            List::new(items).block(
              Block::default()
                .title("Players, Chat - 'tab' to switch")
                .borders(Borders::ALL),
            )
          } else {
            let mut items: Vec<ListItem> = state
              .chat
              .iter()
              .map(|msg| ListItem::new(msg.clone()))
              .collect();
            for _ in items.len() + 3..chunks[1].height.into() {
              items.push(ListItem::new(" "));
            }
            items.push(ListItem::new(if self.chat_selected {
              format!(">{}_", self.chat_buf)
            } else {
              "'ctrl+t' to chat".to_string()
            }));
            List::new(items).block(
              Block::default()
                .title("Chat, Players - 'tab' to switch")
                .borders(Borders::ALL),
            )
          };
          f.render_widget(side, chunks[1]);
        })?;

        if event::poll(Duration::from_secs(0))? {
          if let Event::Key(key) = event::read()? {
            if self.chat_selected {
              match key.code {
                KeyCode::Char(c) => self.chat_buf.push(c),
                KeyCode::Backspace => {
                  self.chat_buf.pop();
                }
                KeyCode::Enter => {
                  bincode::serialize_into(&s, &StateChange::ChatSend(self.chat_buf.clone()))?;
                  self.chat_buf.clear();
                }
                KeyCode::Esc => self.chat_selected = false,
                _ => {}
              }
            } else {
              if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                  KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    process::exit(0);
                  }
                  KeyCode::Char('t') => self.chat_selected = !self.players_open && true,
                  _ => {}
                }
              } else {
                match key.code {
                  KeyCode::Char(c) => bincode::serialize_into(&s, &StateChange::AddLetter(c))?,
                  KeyCode::Backspace => bincode::serialize_into(&s, &StateChange::PopLetter)?,
                  KeyCode::Enter => bincode::serialize_into(&s, &StateChange::Submit)?,
                  KeyCode::Tab => self.players_open = !self.players_open,
                  _ => {}
                }
              }
            }
          }
        }
      }
    });
    loop {
      if let Ok(change) = bincode::deserialize_from(&stream) {
        let mut state = self.state.lock().unwrap();
        match change {
          StateChange::PlayerJoin(i, p) => {
            state.chat.push(format!("{} connected", p.name.clone()));
            state.players.insert(i, p);
          }
          StateChange::PlayerLeave(i) => {
            let p = state.players.get(&i).unwrap().name.clone();
            state.chat.push(format!("{} disconnected", p));
            state.players.remove(&i);
          }
          StateChange::Chat(i, msg) => {
            let p = state.players.get(&i).unwrap().name.clone();
            state.chat.push(format!("{}: {}", p, msg))
          }
          StateChange::AddLetter(c) => {
            let i = state.current_player;
            state.players.get_mut(&i).unwrap().buf.push(c);
          }
          StateChange::PopLetter => {
            let i = state.current_player;
            state.players.get_mut(&i).unwrap().buf.pop();
          }
          StateChange::NextPlayer(i, phrase) => {
            state.current_player = i;
            state.current_phrase = phrase;
            state.players.get_mut(&i).unwrap().buf.clear();
          }
          StateChange::Fail => {
            let i = state.current_player;
            state.players.get_mut(&i).unwrap().buf.clear();
          }
          _ => {}
        }
      }
    }
  }
}
