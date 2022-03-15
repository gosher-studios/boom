use std::{io, env, thread, error, process};
use std::net::{TcpListener, TcpStream, SocketAddrV4, Ipv4Addr};
use std::sync::{Arc, Mutex};
use tui::{Terminal, backend::CrosstermBackend};
use tui::widgets::{Block, Borders, List, ListItem};
use tui::layout::{Layout, Direction, Constraint};
use crossterm::terminal::{
  enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use serde::{Serialize, Deserialize};

type Result<T = ()> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

fn main() -> Result {
  match env::args().nth(1).as_deref() {
    Some("play") => Boom::new().play(),
    Some("host") => Boom::new().host(),
    _ => Err("invalid argument".into()),
  }
}

struct Boom {
  addr: SocketAddrV4,
  state: Arc<Mutex<State>>,
}

#[derive(Serialize, Deserialize)]
struct State {
  players: Vec<String>,
  chat: Vec<(String, String)>,
  max_players: usize,
}

#[derive(Serialize, Deserialize)]
enum StateChange {
  PlayerJoin(String),
  Chat(String, String),
}

impl State {
  fn new() -> Self {
    Self {
      players: vec![],
      chat: vec![],
      max_players: 10,
    }
  }

  fn apply(&mut self, change: StateChange) {
    match change {
      StateChange::PlayerJoin(p) => self.players.push(p),
      StateChange::Chat(p, msg) => self.chat.push((p, msg)),
    }
  }
}

impl Boom {
  fn new() -> Self {
    Self {
      addr: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1234),
      state: Arc::new(Mutex::new(State::new())),
    }
  }

  fn play(self) -> Result {
    let stream = TcpStream::connect(self.addr)?;
    bincode::serialize_into(&stream, "chxry")?;
    let mut state = self.state.lock().unwrap();
    *state = bincode::deserialize_from(&stream)?;
    drop(state);

    let state = self.state.clone();
    thread::spawn(move || -> Result {
      let mut stdout = io::stdout();
      enable_raw_mode()?;
      crossterm::execute!(stdout, EnterAlternateScreen)?;
      let backend = CrosstermBackend::new(stdout);
      let mut terminal = Terminal::new(backend)?;

      loop {
        terminal.draw(|f| {
          let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(f.size());
          let block = Block::default()
            .title("Boom Room - 'ctrl+q' to exit")
            .borders(Borders::ALL);
          f.render_widget(block, chunks[0]);
          let items: Vec<ListItem> = state
            .lock()
            .unwrap()
            .chat
            .iter()
            .map(|(player, msg)| ListItem::new(format!("{}: {}", player, msg)))
            .collect();
          let chat = List::new(items).block(Block::default().title("Chat").borders(Borders::ALL));
          f.render_widget(chat, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
          match key.code {
            KeyCode::Char('q') => {
              if key.modifiers.contains(KeyModifiers::CONTROL) {
                disable_raw_mode()?;
                crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                process::exit(0);
              }
            }
            _ => {}
          }
        }
      }
    });
    loop {
      if let Ok(change) = bincode::deserialize_from(&stream) {
        self.state.lock().unwrap().apply(change);
      }
    }
  }

  fn host(&mut self) -> Result {
    let listener = TcpListener::bind(self.addr)?;
    println!("Listening on port {}", self.addr.port());
    for stream in listener.incoming() {
      let stream = stream?;
      let user: String = bincode::deserialize_from(&stream)?;
      println!("{} connected", user);
      let state = self.state.lock().unwrap();
      bincode::serialize_into(&stream, &*state)?;
      drop(state);
      bincode::serialize_into(
        &stream,
        &StateChange::Chat(String::from("server"), format!("{} connected", user)),
      )?;
    }
    Ok(())
  }
}
