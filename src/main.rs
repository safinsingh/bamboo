extern crate x11rb;

use std::{error::Error, fs};
use x11rb::connection::Connection;

mod lib;
use lib::*;

fn main() -> Result<(), Box<dyn Error>> {
	let read = fs::read_to_string("bamboo.toml")
		.expect("Couldn't find bamboo.toml!");
	let conf: Config =
		toml::from_str(&read).expect("Couldn't deserialize bamboo.toml!");

	let (conn, screen_num) = x11rb::connect(None)?;

	let screen = &conn.setup().roots[screen_num];
	let win = conn.generate_id()?;

	conf.bar.draw(&conn, screen, win)?;

	std::thread::sleep(std::time::Duration::from_secs(10));

	Ok(())
}
