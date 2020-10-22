use serde::Deserialize;
use std::{collections::HashMap, convert::TryInto, error::Error};
use x11rb::{
	connection::Connection, protocol::xproto::*, COPY_DEPTH_FROM_PARENT
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
}

#[derive(Deserialize, Debug)]
pub struct Config {
	pub bar: Bar,
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
		conn.create_window(
			COPY_DEPTH_FROM_PARENT,
			win,
			screen.root,
			((screen.width_in_pixels - self.width) / 2).try_into()?,
			(if self.bottom {
				screen.height_in_pixels - self.height
			} else {
				0
			}) as i16 + self.offset_y,
			self.width,
			self.height,
			self.border_width,
			WindowClass::InputOutput,
			screen.root_visual,
			&Default::default(),
		)?;

		let values =
			ChangeWindowAttributesAux::default().override_redirect(1);
		conn.change_window_attributes(win, &values)?;

		conn.map_window(win)?;

		let _colormap = screen.default_colormap;
		conn.create_colormap(
			ColormapAlloc::All,
			_colormap,
			win,
			screen.root_visual,
		)
		.expect("error creating colormap");

		let win_aux = CreateWindowAux::new()
		.event_mask(EventMask::Exposure | EventMask::StructureNotify | EventMask::NoEvent)
		.background_pixel(screen.white_pixel)
		.win_gravity(Gravity::NorthWest);

		let gc_aux = CreateGCAux::new().foreground(screen.black_pixel);

		conn.flush()?;

		Ok(())
	}
}
