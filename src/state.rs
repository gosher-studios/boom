use std::net::TcpStream;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct State<P> {
  pub players: Vec<P>,
  pub max_players: usize,
  pub chat: Vec<String>,
  pub current_phrase: String,
  pub current_player: usize,
}

#[derive(Serialize, Deserialize)]
pub enum StateChange {
  None,
  PlayerJoin(ClientPlayer),
  PlayerLeave(usize),
  Chat(String, String),
  ChatSend(String),
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
  pub fn new() -> Self {
    Self {
      players: vec![],
      max_players: 10,
      chat: vec![],
      current_phrase: String::new(),
      current_player: 0,
    }
  }
}
