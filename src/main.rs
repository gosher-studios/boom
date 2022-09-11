mod client;
mod menu;
mod server;
mod state;

use std::{env, error};
use client::Client;
use server::Server;

pub type Result<T = ()> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

fn main() -> Result {
  // match env::args().nth(1).as_deref() {
  //   Some("play") => match env::args().nth(2) {
  //     Some(name) => Client::new().play(name),
  //     None => Err("no username".into()),
  //   },
  //   Some("host") => Server::new().host(1234),
  //   _ => Err("invalid argument".into()),
  // }
  menu::menu()
}
