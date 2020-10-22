use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BarConfig {
    width: usize,
    height: usize,
    center: bool,
    #[serde(rename = "offset-x")]
    offset_x: usize,
    #[serde(rename = "offset-y")]
    offset_y: usize,
    widgets: Vec<String>,
    #[serde(rename = "widget-spacing")]
    widget_spacing: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    bar: Vec<BarConfig>,
    widgets: HashMap<String, Widget>,
}

#[derive(Deserialize, Debug)]
pub struct TimeWidget {
    format: Option<String>,
    #[serde(rename = "foreground-normal")]
    foreground_normal: Option<String>,
    #[serde(rename = "background-normal")]
    background_normal: Option<String>,
    #[serde(rename = "foreground-hover")]
    foreground_hover: Option<String>,
    #[serde(rename = "background-hover")]
    background_hover: Option<String>,
}

#[serde(tag = "type")]
#[derive(Deserialize, Debug)]
pub enum Widget {
    #[serde(rename = "time")]
    Time(TimeWidget),
}
