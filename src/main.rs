mod client;
mod server;
mod state;

use std::{env, error};
use client::Client;
use server::Server;

pub type Result<T = ()> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

fn main() -> Result {
  match env::args().nth(1).as_deref() {
    Some("play") => Client::new().play(env::args().nth(2).unwrap()),
    Some("host") => Server::new().host(),
    _ => Err("invalid argument".into()),
  }
}
