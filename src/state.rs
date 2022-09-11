use std::collections::HashMap;
use std::net::TcpStream;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
pub struct State<P> {
  pub players: HashMap<usize, P>,
  pub chat: Vec<String>,
  pub current_phrase: String,
  pub current_player: usize,
  pub timer: DateTime<Utc>,
  /// v settings v
  pub max_players: usize,
  pub timer_length: i64,
  pub time_increase: i64,
  pub lives: u8,
}

#[derive(Serialize, Deserialize)]
pub enum StateChange {
  None,
  PlayerJoin(usize, ClientPlayer),
  PlayerLeave(usize),
  Chat(usize, String),
  ChatSend(String),
  AddLetter(char),
  PopLetter,
  Submit,
  NextPlayer(usize, String),
  Incorrect,
  Fail(usize),
}

#[derive(Serialize, Debug)]
pub struct ServerPlayer {
  pub name: String,
  pub buf: String,
  pub lives: u8,
  #[serde(skip_serializing)]
  pub stream: TcpStream,
}

#[derive(Serialize, Deserialize)]
pub struct ClientPlayer {
  pub name: String,
  pub buf: String,
  pub lives: u8,
}

impl<P> State<P> {
  pub fn new(phrase: String) -> Self {
    Self {
      players: HashMap::new(),
      chat: vec![],
      current_phrase: phrase,
      current_player: 0,
      timer: Utc::now(),
      max_players: 10,
      timer_length: 10,
      time_increase: 1,
      lives: 3,
    }
  }
}

impl ServerPlayer {
  pub fn to_clientplayer(&self) -> ClientPlayer {
    ClientPlayer {
      name: self.name.clone(),
      buf: self.buf.clone(),
      lives: self.lives,
    }
  }
}
