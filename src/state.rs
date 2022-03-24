use std::collections::HashMap;
use std::net::TcpStream;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct State<P> {
  pub players: HashMap<usize, P>,
  pub max_players: usize,
  pub chat: Vec<String>,
  pub current_phrase: String,
  pub current_player: usize,
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
  Fail,
}

#[derive(Serialize, Deserialize)]
pub struct ClientPlayer {
  pub name: String,
  pub buf: String,
}

#[derive(Serialize)]
pub struct ServerPlayer {
  pub name: String,
  pub buf: String,
  #[serde(skip_serializing)]
  pub stream: TcpStream,
}

impl<P> State<P> {
  pub fn new(phrase: String) -> Self {
    Self {
      players: HashMap::new(),
      max_players: 10,
      chat: vec![],
      current_phrase: phrase,
      current_player: 0,
    }
  }
}
