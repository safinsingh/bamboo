use crate::calc::Calculation;
use serde::{
	de::{self, Visitor},
	Deserialize, Deserializer,
};
use std::{collections::HashMap, convert::TryFrom, fmt};

#[serde(rename_all = "kebab-case")]
#[derive(Deserialize, Debug)]
pub struct Bar {
	pub width: Numeric,
	#[serde(skip)]
	pub calc_width: u16,
	pub height: Numeric,
	#[serde(skip)]
	pub calc_height: u16,
	pub center: bool,
	pub bottom: Option<bool>,
	pub border_width: Option<u16>,
	pub offset_x: Option<Numeric>,
	#[serde(skip)]
	pub calc_offset_x: Option<i16>,
	pub offset_y: Option<Numeric>,
	#[serde(skip)]
	pub calc_offset_y: Option<i16>,
	pub widgets: Vec<String>,
	pub widget_spacing: String,
	pub background_color: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
	pub bar: HashMap<String, Bar>,
	pub widgets: HashMap<String, Widget>,
}

#[derive(Clone, Debug)]
pub enum Numeric {
	Number(f32),
	Calculation(Calculation),
}

impl Numeric {
	pub fn get(&self, pc: f32) -> f32 {
		match self {
			Numeric::Number(v) => *v,
			Numeric::Calculation(calc) => calc.calculate(pc),
		}
	}
}

impl<'de> Deserialize<'de> for Numeric {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		macro_rules! visit_number {
			($n:ident, $t:ty) => {
				fn $n<E>(self, v: $t) -> Result<Self::Value, E>
				where
					E: de::Error,
				{
					Ok(Numeric::Number(v as f32))
				}
			};
		}

		macro_rules! visit_string {
			($n:ident, $t:ty) => {
				fn $n<E>(self, v: $t) -> Result<Self::Value, E>
				where
					E: de::Error,
				{
					Ok(Numeric::Calculation(
						Calculation::try_from(v).map_err(|e| {
							de::Error::custom(&format!("{}", e))
						})?,
					))
				}
			};
		}

		struct NumericVisitor;

		impl<'de> Visitor<'de> for NumericVisitor {
			type Value = Numeric;

			visit_number!(visit_i8, i8);

			visit_number!(visit_u8, u8);

			visit_number!(visit_i16, i16);

			visit_number!(visit_u16, u16);

			visit_number!(visit_i32, i32);

			visit_number!(visit_u32, u32);

			visit_number!(visit_f32, f32);

			visit_number!(visit_i64, i64);

			visit_number!(visit_u64, u64);

			visit_number!(visit_f64, f64);

			visit_string!(visit_str, &str);

			visit_string!(visit_string, String);

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, "Expecting a decimal, integer, or string.")
			}
		}

		deserializer.deserialize_identifier(NumericVisitor)
	}
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
