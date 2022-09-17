mod client;
mod menu;
mod server;
mod state;

use std::error;

pub type Result<T = ()> = std::result::Result<T, Box<dyn error::Error + Send + Sync>>;

fn main() -> Result {
  menu::menu()
}
