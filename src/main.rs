use std::{io, env, thread, error, process};
use std::time::Duration;
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
    Some("play") => Game::new().play(env::args().nth(2).unwrap()),
    Some("host") => Server::new().host(),
    _ => Err("invalid argument".into()),
  }
}

struct Game {
  addr: SocketAddrV4,
  state: Arc<Mutex<State<String>>>,
}

#[derive(Serialize)]
struct ServerPlayer {
  name: String,
  #[serde(skip_serializing)]
  stream: TcpStream,
}

#[derive(Clone)]
struct Server {
  state: Arc<Mutex<State<ServerPlayer>>>,
}

#[derive(Serialize, Deserialize)]
struct State<P> {
  players: Vec<P>,
  chat: Vec<(String, String)>,
  max_players: usize,
}

#[derive(Serialize, Deserialize)]
enum StateChange {
  None,
  PlayerJoin(String),
  PlayerLeave(usize),
  Chat(String, String),
}

impl<P> State<P> {
  fn new() -> Self {
    Self {
      players: vec![],
      chat: vec![],
      max_players: 10,
    }
  }
}

impl State<String> {
  fn apply(&mut self, change: StateChange) {
    match change {
      StateChange::None => {}
      StateChange::PlayerJoin(p) => self.players.push(p),
      StateChange::PlayerLeave(i) => {
        self.players.remove(i);
      }
      StateChange::Chat(p, msg) => self.chat.push((p, msg)),
    }
  }
}

impl Game {
  fn new() -> Self {
    Self {
      addr: SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1234),
      state: Arc::new(Mutex::new(State::new())),
    }
  }

  fn play(self, name: String) -> Result {
    let stream = TcpStream::connect(self.addr)?;
    bincode::serialize_into(&stream, &name)?;
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

        if event::poll(Duration::from_secs(0))? {
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
      }
    });
    loop {
      if let Ok(change) = bincode::deserialize_from(&stream) {
        self.state.lock().unwrap().apply(change);
      }
    }
  }
}

impl Server {
  fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(State::new())),
    }
  }

  fn host(self) -> Result {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1234);
    let listener = TcpListener::bind(addr)?;
    println!("Listening on port {}", addr.port());
    for stream in listener.incoming() {
      let stream = stream?;
      let s = self.clone();
      thread::spawn(move || s.handle_player(stream));
    }
    Ok(())
  }

  fn handle_player(&self, stream: TcpStream) -> Result {
    let name: String = bincode::deserialize_from(&stream)?;
    println!("{} connected", name);
    let mut state = self.state.lock().unwrap();
    bincode::serialize_into(&stream, &*state)?;
    state.players.push(ServerPlayer {
      name: name.clone(),
      stream: stream.try_clone()?,
    });
    drop(state);
    self.broadcast(StateChange::PlayerJoin(name.clone()))?;
    self.broadcast(StateChange::Chat(
      String::from("server"),
      format!("{} connected", name),
    ))?;
    loop {
      thread::sleep(Duration::from_secs(2));
      self.broadcast(StateChange::None)?;
    }
  }

  fn broadcast(&self, change: StateChange) -> Result {
    let mut disconnects = vec![];
    let mut i = 0;
    self.state.lock().unwrap().players.retain(|player| {
      let o = bincode::serialize_into(&player.stream, &change).is_ok();
      if !o {
        disconnects.push((player.name.clone(), i));
      }
      i += 1;
      o
    });
    for (name, i) in disconnects {
      self.broadcast(StateChange::Chat(
        String::from("server"),
        format!("{} disconnected", name),
      ))?;
      self.broadcast(StateChange::PlayerLeave(i))?;
      println!("{} disconnected", name);
    }
    Ok(())
  }
}
