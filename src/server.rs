use std::thread;
use std:: sync::atomic::{AtomicUsize,Ordering};
use std::net::{TcpListener, TcpStream, SocketAddrV4, Ipv4Addr};
use std::sync::{Arc, Mutex};

use crate::state::{State, StateChange, ServerPlayer, ClientPlayer};
use crate::Result;

static ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
pub struct Server {
  state: Arc<Mutex<State<ServerPlayer>>>,
  words: Arc<Vec<String>>,
  phrases: Arc<Vec<String>>
}

impl Server {
  pub fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(State::new())),
      words: Arc::new(serde_json::from_str(include_str!("words.json")).unwrap()),
      phrases: Arc::new(serde_json::from_str(include_str!("phrases.json")).unwrap())
    }
  }

  pub fn host(self) -> Result {
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
    let id = ID.fetch_add(1,Ordering::Relaxed);
    println!("{} connected", name);
    let player = ServerPlayer {
      id,
      name: name.clone(),
      buf: String::new(),
      stream: stream.try_clone()?,
    };
    let mut state = self.state.lock().unwrap();
    bincode::serialize_into(&stream, &*state)?;
    state.players.push(player);
    drop(state);
    self.broadcast(StateChange::PlayerJoin(ClientPlayer {
      id,
      name: name.clone(),
      buf: String::new(),
    }))?;

    loop {
      if let Ok(change) = bincode::deserialize_from(&stream) {
        match change {
          StateChange::ChatSend(msg) => {
            if !msg.trim().is_empty() {
              println!("{}: {}", name.clone(), msg);
              self.broadcast(StateChange::Chat(name.clone(), msg))?;
            }
          }
          StateChange::AddLetter(c) => {
            if c.is_alphabetic() {
              //check if currentplayer
              // what the fuck

              let mut state = self.state.lock().unwrap();
              let player = state.players.iter_mut().find(|x| x.id == id).unwrap();
              if id == player.id{
                player.buf.push(c);
                self.broadcast(StateChange::AddLetter(c))?;
              };
            }
          }
          StateChange::PopLetter => {
            let mut state = self.state.lock().unwrap();
            let player = state.players.iter_mut().find(|x| x.id == id).unwrap();
            if id == player.id {
              player.buf.pop();
              self.broadcast(StateChange::PopLetter)?;
            }
            
          }
          _ => {}
        }
      }
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
      self.broadcast(StateChange::PlayerLeave(i))?;
      println!("{} disconnected", name);
    }
    Ok(())
  }
}
