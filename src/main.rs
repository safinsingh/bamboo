extern crate x11rb;

use anyhow::{anyhow, Context, Result};
use clap::Clap;
use std::{
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

fn main() -> Result<()> {
	let opts: Opts = Opts::parse();
	let read = fs::read_to_string(&opts.config).with_context(|| {
		format!("Failed to read configuration file from: {}", opts.config)
	})?;
	let conf: Config = toml::from_str(&read)
		.with_context(|| "Failed to deserialize configuration")?;

	let (conn, screen_num) = x11rb::connect(None)
		.with_context(|| "Failed to initialize connection to X server")?;
	let screen = &conn.setup().roots[screen_num];
	let win = conn
		.generate_id()
		.with_context(|| "Failed to generate new X11 ID for bar")?;

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
			.ok_or(anyhow!("Could not find bar: {}", opts.bar))?;
		bar.draw(&conn, screen, win).with_context(|| {
			format!("Error encountered while drawing bar: {}", opts.bar)
		})?;
	}

	Ok(())
}
