use serde::Deserialize;
use std::collections::HashMap;

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Debug)]
pub struct Bar {
	pub width: u16,
	pub height: u16,
	pub center: bool,
	pub bottom: Option<bool>,
	pub border_width: Option<u16>,
	pub offset_x: Option<i16>,
	pub offset_y: Option<i16>,
	pub widgets: Vec<String>,
	pub widget_spacing: String,
	pub background_color: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
	pub bar: HashMap<String, Bar>,
	pub widgets: HashMap<String, Widget>,
}

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Debug)]
pub struct TextWidget {
	pub text: String,
	pub font: Option<String>,
	pub font_size: Option<f64>,
	pub font_style: Option<FontStyle>,
	pub color: String,
}

#[derive(Deserialize, Debug)]
pub struct FontStyle {
	pub weight: String,
	pub slant: String,
}

#[serde(tag = "type")]
#[derive(Deserialize, Debug)]
pub enum Widget {
	#[serde(rename = "text")]
	Text(TextWidget),
}
