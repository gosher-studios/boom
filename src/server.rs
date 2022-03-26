use std::thread;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, SocketAddrV4, Ipv4Addr};
use std::sync::{Arc, Mutex};
use rand::seq::SliceRandom;
use chrono::{Utc, Duration};
use crate::state::{State, StateChange, ServerPlayer};
use crate::Result;

#[derive(Clone)]
pub struct Server {
  state: Arc<Mutex<State<ServerPlayer>>>,
  words: Arc<Vec<String>>,
  phrases: Arc<Vec<String>>,
  usedwords: Arc<Mutex<Vec<String>>>,
}

impl Server {
  pub fn new() -> Self {
    let phrases: Vec<String> = serde_json::from_str(include_str!("phrases.json")).unwrap();
    Self {
      state: Arc::new(Mutex::new(State::new(
        phrases.choose(&mut rand::thread_rng()).unwrap().to_string(),
      ))),
      words: Arc::new(serde_json::from_str(include_str!("words.json")).unwrap()),
      phrases: Arc::new(phrases),
      usedwords: Arc::new(Mutex::new(vec![]))
    }
  }

  pub fn host(self) -> Result {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1234);
    let listener = TcpListener::bind(addr)?;
    println!("Listening on port {}", addr.port());
    let s = self.clone();
    thread::spawn(move || s.game_manager());
    for (id, stream) in listener.incoming().enumerate() {
      let stream = stream?;
      let s = self.clone();
      thread::spawn(move || s.handle_player(stream, id));
    }
    Ok(())
  }

  fn game_manager(&self) -> Result {
    loop {
      let mut state = self.state.lock().unwrap();
      if Utc::now() - state.timer > Duration::seconds(state.timer_length) {
        let i = state.current_player;
        state.timer = Utc::now();
        state.players.get_mut(&i).unwrap().lives -= 1;
        let next = next_player(&state.players, state.current_player);
        state.current_player = next;
        drop(state);
        self.broadcast(StateChange::Fail(next))?;
      } else {
        drop(state);
      }
      self.broadcast(StateChange::None)?;
      thread::sleep(Duration::milliseconds(10).to_std()?);
    }
  }

  fn handle_player(&self, stream: TcpStream, id: usize) -> Result {
    let name: String = bincode::deserialize_from(&stream)?;
    println!("{} connected", name);
    let mut state = self.state.lock().unwrap();
    let player = ServerPlayer {
      name: name.clone(),
      buf: String::new(),
      lives: state.lives,
      stream: stream.try_clone()?,
    };
    let cplayer = player.to_clientplayer();
    bincode::serialize_into(&stream, &id)?;
    bincode::serialize_into(&stream, &*state)?;
    state.players.insert(id, player);
    drop(state);
    self.broadcast(StateChange::PlayerJoin(id, cplayer))?;

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
            let mut state = self.state.lock().unwrap();
            if id == state.current_player {
              let buf = &state.players.get(&id).unwrap().buf;
              let used = self.usedwords.lock().unwrap();
              if buf.contains(&state.current_phrase) && self.words.contains(buf) && !used.contains(&buf) {
                used.push(buf.to_string());
                let phrase = self
                  .phrases
                  .choose(&mut rand::thread_rng())
                  .unwrap()
                  .to_string();
                let next = next_player(&state.players, state.current_player);
                state.current_player = next;
                state.current_phrase = phrase.clone();
                state.players.get_mut(&next).unwrap().buf.clear();
                state.timer = state.timer + Duration::seconds(state.time_increase);
                drop(state);
                self.broadcast(StateChange::NextPlayer(next, phrase))?;
              } else {
                state.players.get_mut(&id).unwrap().buf.clear();
                drop(state);
                self.broadcast(StateChange::Incorrect)?;
              }
            }
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

fn next_player(players: &HashMap<usize, ServerPlayer>, current: usize) -> usize {
  let mut iter = players.iter().filter(|(_,p)| p.lives > 0);
  // todo no players left also 1 player left !!!
  let i = iter.position(|(id, _)| *id == current).unwrap();
  match iter.nth(i + 1) {
    Some(i) => *i.0,
    None => 0,
  }
}
