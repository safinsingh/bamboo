extern crate x11rb;

use std::{collections::HashMap, error::Error, fs};
use x11rb::{
	connection::Connection, protocol::xproto::*, COPY_DEPTH_FROM_PARENT,
};

mod lib;
use lib::*;

fn main() -> Result<(), Box<dyn Error>> {
	let read =
		fs::read_to_string("bamboo.toml").expect("Couldn't find bamboo.toml!");
	let conf: Config =
		toml::from_str(&read).expect("Couldn't deserialize bamboo.toml!");

	println!("{:?}", conf);

	let (conn, screen_num) = x11rb::connect(None)?;

	let screen = &conn.setup().roots[screen_num];
	let win = conn.generate_id()?;

	conn.create_window(
		COPY_DEPTH_FROM_PARENT,
		win,
		screen.root,
		(screen.width_in_pixels - conf.bar.width) / 2,
		(if conf.bar.bottom {
			screen.height_in_pixels - conf.bar.height
		} else {
			0
		}) + conf.bar.offset_y,
		conf.bar.width,
		conf.bar.height,
		conf.bar.border,
		WindowClass::InputOutput,
		screen.root_visual,
		&Default::default(),
	)?;

	conn.map_window(win)?;
	conn.flush()?;

	let _colormap = screen.default_colormap;
	create_colormap(
		&conn,
		ColormapAlloc::All,
		_colormap,
		win,
		screen.root_visual,
	)
	.expect("error creating colormap");

	std::thread::sleep(std::time::Duration::from_secs(10));

	Ok(())
}
