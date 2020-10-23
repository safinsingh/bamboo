use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, convert::TryInto, u32};
use x11rb::{
	connection::Connection,
	protocol::xproto::{ConnectionExt, Rectangle, *},
	wrapper::ConnectionExt as _,
	COPY_DEPTH_FROM_PARENT,
};

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
	) -> Result<()> {
		let root = screen.root;
		let root_sz = (screen.width_in_pixels, screen.height_in_pixels);
		let x_pos = ((root_sz.0 - self.width) / 2)
			.try_into()
			.with_context(|| "Failed to set X position of bar")?;
		let y_pos = (if self.bottom {
			root_sz.1 - self.height
		} else {
			0
		}) as i16 + self.offset_y; // FIX: use try_into here
		let bg_color = u32::from_str_radix(
			self.background_normal.trim_start_matches('#'),
			16,
		)
		.with_context(|| {
			"Failed to convert bar background color to u32"
		})?;

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
		)
		.with_context(|| "Failed to create bar window")?;

		// override default wm decorations
		let values =
			ChangeWindowAttributesAux::default().override_redirect(1);
		conn.change_window_attributes(win, &values).with_context(
			|| "Failed to set bar window attributes to override redirect",
		)?;

		conn.map_window(win)
			.with_context(|| "Failed to map main bar window to root")?;

		let pixmap = conn.generate_id().with_context(|| {
			"Failed to generate new X11 ID for bar pixmap"
		})?;
		conn.create_pixmap(
			screen.root_depth,
			pixmap,
			root,
			self.width,
			self.height,
		)
		.with_context(|| "Failed to create bar pixmap")?;

		let gc = conn.generate_id().with_context(|| {
			"Failed to generate new X11 ID for bar graphics context"
		})?;
		let gc_aux = CreateGCAux::new().foreground(bg_color);
		conn.create_gc(gc, root, &gc_aux).with_context(|| {
			"Failed to create graphics context on root drawable"
		})?;

		let rect = Rectangle {
			x: 0,
			y: 0,
			width: self.width,
			height: self.height,
		};

		conn.flush()
			.with_context(|| "Failed to flush X11 connection")?;

		// fill gc with rectangle spanning entire w/h
		conn.poly_fill_rectangle(pixmap, gc, &[rect]).with_context(
			|| "Failed to fill background rectangle on bar",
		)?;

		// draw pixmap on window
		conn.change_window_attributes(
			win,
			&ChangeWindowAttributesAux::new().background_pixmap(pixmap),
		)
		.with_context(|| {
			"Failed to assign background pixmap to bar window"
		})?;

		conn.clear_area(false, win, 0, 0, 0, 0)
			.with_context(|| "Failed to clear window area")?;

		// destroy pixmap and gc
		conn.free_pixmap(pixmap)
			.with_context(|| "Failed to destroy bar pixmap")?;
		conn.free_gc(gc)
			.with_context(|| "Failed to destroy bar graphics context")?;

		conn.sync()
			.with_context(|| "Failed to sync connection to X11 server")?;

		Ok(())
	}
}
