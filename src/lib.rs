use serde::Deserialize;
use std::{collections::HashMap, convert::TryInto, error::Error};
use x11rb::{
	connection::Connection,
	protocol::xproto::{ConnectionExt, Rectangle, *},
	wrapper::ConnectionExt as _,
	COPY_DEPTH_FROM_PARENT,
};

use std::u32;

#[derive(Deserialize, Debug)]
pub struct Bar {
	pub width: u16,
	pub height: u16,
	pub center: bool,
	#[serde(default = "default_false")]
	pub bottom: bool,
	#[serde(rename = "border-width", default = "default_zero")]
	pub border_width: u16,
	#[serde(rename = "offset-x")]
	pub offset_x: i16,
	#[serde(rename = "offset-y")]
	pub offset_y: i16,
	pub widgets: Vec<String>,
	#[serde(rename = "widget-spacing")]
	pub widget_spacing: String,
	#[serde(rename = "foreground-normal", default = "default_black")]
	pub foreground_normal: String,
	#[serde(rename = "background-normal", default = "default_white")]
	pub background_normal: String,
	#[serde(rename = "foreground-hover", default = "default_black")]
	pub foreground_hover: String,
	#[serde(rename = "background-hover", default = "default_white")]
	pub background_hover: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
	pub bar: HashMap<String, Bar>,
	pub widgets: HashMap<String, Widget>,
}

#[derive(Deserialize, Debug)]
pub struct TimeWidget {
	#[serde(default = "default_time")]
	pub format: String,
	#[serde(rename = "foreground-normal", default = "default_black")]
	pub foreground_normal: String,
	#[serde(rename = "background-normal", default = "default_white")]
	pub background_normal: String,
	#[serde(rename = "foreground-hover", default = "default_black")]
	pub foreground_hover: String,
	#[serde(rename = "background-hover", default = "default_white")]
	pub background_hover: String,
}

#[serde(tag = "type")]
#[derive(Deserialize, Debug)]
pub enum Widget {
	#[serde(rename = "time")]
	Time(TimeWidget),
}

fn default_false() -> bool {
	false
}
fn default_zero() -> u16 {
	0
}
fn default_white() -> String {
	"#ffffff".into()
}
fn default_black() -> String {
	"#000000".into()
}
fn default_time() -> String {
	"%H:%M".into()
}

impl Bar {
	pub fn draw(
		&self,
		conn: &(impl Connection + Send + Sync),
		screen: &Screen,
		win: Window,
	) -> Result<(), Box<dyn Error>> {
		let root = screen.root;
		let root_sz = (screen.width_in_pixels, screen.height_in_pixels);
		let x_pos = ((root_sz.0 - self.width) / 2).try_into()?;
		let y_pos = (if self.bottom {
			root_sz.1 - self.height
		} else {
			0
		}) as i16 + self.offset_y;
		let bg_color = u32::from_str_radix(
			self.background_normal.trim_start_matches('#'),
			16,
		)?;

		conn.create_window(
			COPY_DEPTH_FROM_PARENT,   // window depth
			win,                      // window id
			root,                     // parent window
			x_pos,                    // x position
			y_pos,                    // y position
			self.width,               // width
			self.height,              // height
			self.border_width,        // border width
			WindowClass::InputOutput, // window class
			screen.root_visual,       // visual
			&Default::default(),      // value list
		)?;

		// override default wm decorations
		let values =
			ChangeWindowAttributesAux::default().override_redirect(1);
		conn.change_window_attributes(win, &values)?;

		conn.map_window(win)?;

		let pixmap = conn.generate_id()?;
		conn.create_pixmap(
			screen.root_depth,
			pixmap,
			root,
			self.width,
			self.height,
		)?;

		let gc = conn.generate_id().unwrap();
		let gc_aux = CreateGCAux::new().foreground(bg_color);
		conn.create_gc(gc, root, &gc_aux)?;

		let rect = Rectangle {
			x: 0,
			y: 0,
			width: self.width,
			height: self.height,
		};

		conn.flush()?;

		// fill gc with rectangle spanning entire w/h
		conn.poly_fill_rectangle(pixmap, gc, &[rect])?;

		// draw pixmap on window
		conn.change_window_attributes(
			win,
			&ChangeWindowAttributesAux::new().background_pixmap(pixmap),
		)?;

		conn.clear_area(false, win, 0, 0, 0, 0)?;

		// destroy pixmap and gc
		conn.free_pixmap(pixmap)?;
		conn.free_gc(gc)?;

		conn.sync()?;

		Ok(())
	}
}
