use std::thread;
use std::net::{TcpListener, TcpStream, SocketAddrV4, Ipv4Addr};
use std::sync::{Arc, Mutex};

use crate::state::{State, StateChange, ServerPlayer, ClientPlayer};
use crate::Result;

#[derive(Clone)]
pub struct Server {
  state: Arc<Mutex<State<ServerPlayer>>>,
  words: Arc<Vec<String>>,
  phrases: Arc<Vec<String>>,
}

impl Server {
  pub fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(State::new())),
      words: Arc::new(serde_json::from_str(include_str!("words.json")).unwrap()),
      phrases: Arc::new(serde_json::from_str(include_str!("phrases.json")).unwrap()),
    }
  }

  pub fn host(self) -> Result {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1234);
    let listener = TcpListener::bind(addr)?;
    println!("Listening on port {}", addr.port());
    for (id, stream) in listener.incoming().enumerate() {
      let stream = stream?;
      let s = self.clone();
      thread::spawn(move || s.handle_player(stream, id));
    }
    Ok(())
  }

  fn handle_player(&self, stream: TcpStream, id: usize) -> Result {
    let name: String = bincode::deserialize_from(&stream)?;
    println!("{} connected", name);
    let player = ServerPlayer {
      name: name.clone(),
      buf: String::new(),
      stream: stream.try_clone()?,
    };
    let mut state = self.state.lock().unwrap();
    bincode::serialize_into(&stream, &*state)?;
    state.players.insert(id, player);
    drop(state);
    self.broadcast(StateChange::PlayerJoin(
      id,
      ClientPlayer {
        name: name.clone(),
        buf: String::new(),
      },
    ))?;

    loop {
      if let Ok(change) = bincode::deserialize_from(&stream) {
        match change {
          StateChange::ChatSend(msg) => {
            if !msg.trim().is_empty() {
              println!("{}: {}", name.clone(), msg);
              self.broadcast(StateChange::Chat(id, msg))?;
            }
          }
          StateChange::AddLetter(c) => {
            if c.is_alphabetic() {
              let mut state = self.state.lock().unwrap();
              if id == state.current_player {
                state.players.get_mut(&id).unwrap().buf.push(c);
                drop(state);
                self.broadcast(StateChange::AddLetter(c))?;
              }
            }
          }
          StateChange::PopLetter => {
            let mut state = self.state.lock().unwrap();
            if id == state.current_player {
              state.players.get_mut(&id).unwrap().buf.pop();
              drop(state);
              self.broadcast(StateChange::PopLetter)?;
            }
          }
          StateChange::Submit => {
            // let mut state = self.state.lock().unwrap();

            // if id == state.current_player {
            //   let player = state.players.iter_mut().find(|x| x.id == id).unwrap();
            //   if self.words.contains(&player.buf) {
            //     //drop(state);
            //     println!("wooo you did it it is real");
            //     player.buf.clear();
            //     //TODO: PUT INTO IF STATEMENT TO CHECK IF CURRENTPLAYER BECOMES NOT VALID ID
            //     state.current_player += 1;
            //     //TODO: broadcast later fucming retard
            //     //self.broadcast(StateChange::Submit)?;
            //   }
            // }
          }
          _ => {}
        }
      }
    }
  }

  fn broadcast(&self, change: StateChange) -> Result {
    let mut disconnects = vec![];
    self.state.lock().unwrap().players.retain(|i, player| {
      let o = bincode::serialize_into(&player.stream, &change).is_ok();
      if !o {
        disconnects.push((player.name.clone(), *i));
      }
      o
    });
    for (name, i) in disconnects {
      self.broadcast(StateChange::PlayerLeave(i))?;
      println!("{} disconnected", name);
    }
    Ok(())
  }
}
