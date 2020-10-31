use regex::Regex;
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct Calculation(Value, Vec<Segment>);

impl Calculation {
	pub fn calculate(&self, pc: f32) -> f32 {
		let mut to_return = match (self.0).1 {
			Unit::Pixel | Unit::None => (self.0).0,
			Unit::Percent => ((self.0).0 / 100.0) * pc,
		};
		for segment in (self.1).iter() {
			macro_rules! calculation {
				($expr:expr) => { $expr };
				($op:tt) => {
					match (segment.1).1 {
						Unit::Pixel | Unit::None => calculation!(to_return $op (segment.1).0),
						Unit::Percent => calculation!(to_return $op ((segment.1).0 / 100.0) * pc),
					}
				};
			}
			match segment.0 {
				Operation::Add => calculation!(+=),
				Operation::Subtract => calculation!(-=),
				Operation::Multiply => calculation!(*=),
				Operation::Divide => calculation!(/=),
				Operation::Remainder => calculation!(%=),
				Operation::Exponent => match (segment.1).1 {
					Unit::Pixel | Unit::None => {
						to_return = to_return.powf((segment.1).0)
					}
					Unit::Percent => {
						to_return =
							to_return.powf(((segment.1).0 / 100.0) * pc)
					}
				},
			}
		}
		to_return
	}
}

impl TryFrom<&str> for Calculation {
	type Error = String;

	fn try_from(v: &str) -> Result<Self, String> {
		let regex = Regex::new(r#"(?:(\d+)(%|px|))(.*?)$"#).unwrap();
		let caps = regex
			.captures(v)
			.ok_or_else(|| String::from("Invalid syntax"))?;
		let base =
			caps.get(1)
				.unwrap()
				.as_str()
				.parse::<f32>()
				.map_err(|err| {
					format!(
						"Failed to parse \"{}\" into an f32: {}",
						caps.get(1).unwrap().as_str(),
						err
					)
				})?;
		let unit = Unit::try_from(caps.get(2).unwrap().as_str())?;
		let segments = {
			let mut to_return = Vec::new();
			let as_str = caps.get(3).unwrap().as_str();
			let as_split = as_str.split(':').collect::<Vec<&str>>();
			for segment_as_str in as_split.iter().skip(1) {
				to_return.push(Segment::try_from(*segment_as_str)?);
			}
			to_return
		};
		Ok(Calculation(Value(base, unit), segments))
	}
}

impl TryFrom<String> for Calculation {
	type Error = String;

	fn try_from(v: String) -> Result<Self, String> {
		Self::try_from(v.as_str())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Segment(Operation, Value);

impl TryFrom<&str> for Segment {
	type Error = String;

	fn try_from(v: &str) -> Result<Self, Self::Error> {
		let regex =
			Regex::new(r#"(?:(\+|-|\*|/|%|^|.)(\d+(?:\.\d+)?)(%|px|))$"#)
				.unwrap();
		let caps = regex
			.captures(v)
			.ok_or(format!("Invalid segment: \"{}\"", v))?;
		let operation = Operation::try_from(caps.get(1).unwrap().as_str())?;
		let value =
			caps.get(2)
				.unwrap()
				.as_str()
				.parse::<f32>()
				.map_err(|err| {
					format!(
						"Failed to parse \"{}\" into an f32: {}",
						caps.get(2).unwrap().as_str(),
						err
					)
				})?;
		let unit = Unit::try_from(caps.get(3).unwrap().as_str())?;
		Ok(Segment(operation, Value(value, unit)))
	}
}

impl TryFrom<String> for Segment {
	type Error = String;

	fn try_from(v: String) -> Result<Self, Self::Error> {
		Self::try_from(v.as_str())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
	Add,
	Subtract,
	Multiply,
	Divide,
	Exponent,
	Remainder,
}

impl TryFrom<&str> for Operation {
	type Error = String;

	fn try_from(v: &str) -> Result<Self, Self::Error> {
		match v {
			"+" => Ok(Operation::Add),
			"-" => Ok(Operation::Subtract),
			"*" => Ok(Operation::Multiply),
			"/" => Ok(Operation::Divide),
			"%" => Ok(Operation::Remainder),
			"^" => Ok(Operation::Exponent),
			_ => Err(format!(
				"Invalid operation \"{src}\". Expected +, -, *, /, or %.",
				src = v
			)),
		}
	}
}

impl TryFrom<String> for Operation {
	type Error = String;

	fn try_from(v: String) -> Result<Self, Self::Error> {
		Self::try_from(v.as_str())
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct Value(f32, Unit);

#[derive(Clone, Debug, PartialEq)]
pub enum Unit {
	Pixel,
	Percent,
	None,
}

impl TryFrom<&str> for Unit {
	type Error = String;

	fn try_from(v: &str) -> Result<Self, Self::Error> {
		match v {
			"%" => Ok(Unit::Percent),
			"px" => Ok(Unit::Pixel),
			"" => Ok(Unit::None),
			_ => Err(format!(
				"Invalid unit \"{src}\". Expected % or px.",
				src = v
			)),
		}
	}
}

impl TryFrom<String> for Unit {
	type Error = String;

	fn try_from(v: String) -> Result<Self, Self::Error> {
		Self::try_from(v.as_str())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse() {
		assert_eq!(
			Calculation::try_from("100%:-50px").unwrap(),
			Calculation(
				Value(100.0, Unit::Percent),
				vec![Segment(Operation::Subtract, Value(50.0, Unit::Pixel),)],
			)
		);
	}

	#[test]
	fn calculate() {
		assert_eq!(
			Calculation::try_from("100%:-50px")
				.unwrap()
				.calculate(100.0)
				.round(),
			50.0
		);
		assert_eq!(
			Calculation::try_from("50%:-25px")
				.unwrap()
				.calculate(100.0)
				.round(),
			25.0
		);
		assert_eq!(
			Calculation::try_from("100px:-50px")
				.unwrap()
				.calculate(0.0)
				.round(),
			50.0
		);
		assert_eq!(
			Calculation::try_from("100%:+50px")
				.unwrap()
				.calculate(100.0)
				.round(),
			150.0
		);
		assert_eq!(
			Calculation::try_from("100%:*2")
				.unwrap()
				.calculate(100.0)
				.round(),
			200.0
		);
		assert_eq!(
			Calculation::try_from("100%:/2")
				.unwrap()
				.calculate(100.0)
				.round(),
			50.0
		);
		assert_eq!(
			Calculation::try_from("100%:^2")
				.unwrap()
				.calculate(4.0)
				.round(),
			16.0
		);
		assert_eq!(
			Calculation::try_from("100%:%2")
				.unwrap()
				.calculate(10.0)
				.round(),
			0.0
		);
	}
}
