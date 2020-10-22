extern crate x11rb;

use clap::Clap;
use std::{
	error::Error,
	fs,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};
use x11rb::connection::Connection;

mod lib;
use lib::*;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "safinsingh <safin.singh@gmail.com>")]
struct Opts {
	/// Location of configuration file
	#[clap(short, long, default_value = "bamboo.toml")]
	config: String,

	/// Name of bar to display
	#[clap(short, long, default_value = "default")]
	bar: String,
}

fn main() -> Result<(), Box<dyn Error>> {
	let opts: Opts = Opts::parse();
	let read = fs::read_to_string(opts.config)?;
	let conf: Config = toml::from_str(&read)?;

	let (conn, screen_num) = x11rb::connect(None)?;
	let screen = &conn.setup().roots[screen_num];
	let win = conn.generate_id()?;

	let running = Arc::new(AtomicBool::new(true));
	let r = running.clone();

	ctrlc::set_handler(move || {
		r.store(false, Ordering::SeqCst);
	})
	.expect("Error setting Ctrl-C handler");

	while running.load(Ordering::SeqCst) {
		let bar = conf
			.bar
			.get(&opts.bar)
			.ok_or(format!("Could not find bar: {}", opts.bar))?;
		bar.draw(&conn, screen, win)?;
	}

	Ok(())
}
