#![feature(bool_to_option)]

extern crate notify;
extern crate x11rb;
extern crate xcb;

use anyhow::{anyhow, Context};
use clap::Clap;
use std::{
	fs,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
};
use x11rb::{connection::Connection, xcb_ffi::XCBConnection};

mod conf;
use conf::*;
mod bar;
mod calc;

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

fn main() -> anyhow::Result<()> {
	let opts: Opts = Opts::parse();
	let read = fs::read_to_string(&opts.config).with_context(|| {
		format!("Failed to read configuration file from: {}", opts.config)
	})?;

	let mut conf: Config = toml::from_str(&read)
		.with_context(|| "Failed to deserialize configuration")?;

	let (xcb_conn, screen_num) = xcb::Connection::connect(None)
		.with_context(|| "Failed to initialize connection to X server")?;
	let screen_num = screen_num as usize;
	let conn = unsafe {
		XCBConnection::from_raw_xcb_connection(
			xcb_conn.get_raw_conn() as _,
			false,
		)
		.unwrap()
	};

	let screen = &conn.setup().roots[screen_num];
	let win = conn
		.generate_id()
		.with_context(|| "Failed to generate new X11 ID for bar")?;

	let running = Arc::new(AtomicBool::new(true));
	let r = running.clone();
	let mut ran = false;

	ctrlc::set_handler(move || {
		r.store(false, Ordering::SeqCst);
	})
	.with_context(|| "Error setting Ctrl-C handler")?;

	while running.load(Ordering::SeqCst) {
		if !ran {
			let bar = conf
				.bar
				.get_mut(&opts.bar)
				.ok_or_else(|| anyhow!("Could not find bar: {}", opts.bar))?;

			bar.calc(screen);

			bar.draw(&xcb_conn, &conn, screen, win).with_context(|| {
				format!("Error encountered while drawing bar: {}", opts.bar)
			})?;

			ran = true;
		}
		std::thread::sleep(std::time::Duration::from_millis(16));
	}

	Ok(())
}
