use serde::Deserialize;
use std::collections::HashMap;

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

fn default_false() -> bool { false }
fn default_zero() -> u16 { 0 }
fn default_white() -> String { "#ffffff".into() }
fn default_black() -> String { "#000000".into() }
fn default_time() -> String { "%H:%M".into() }
