extern crate x11rb;

use std::error::Error;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::COPY_DEPTH_FROM_PARENT;

#[derive(Deserialize, Debug)]
struct BarConfig {
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
struct Config {
    bar: Vec<BarConfig>,
    widgets: HashMap<String, Widget>,
}

#[derive(Deserialize, Debug)]
struct TimeWidget {
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
enum Widget {
    #[serde(rename = "time")]
    Time(TimeWidget),
}

fn main() -> Result<(), Box<dyn Error>> {
	let read = fs::read_to_string("bamboo.toml").expect("Couldn't find bamboo.toml!");
    let conf: Config = toml::from_str(&read).expect("Couldn't deserialize bamboo.toml!");

    println!("{:?}", conf);

    // Open the connection to the X server. Use the DISPLAY environment variable.
    let (conn, screen_num) = x11rb::connect(None)?;

    // Get the screen #screen_num
    let screen = &conn.setup().roots[screen_num];


    // Ask for our window's Id
    let win = conn.generate_id()?;

    // Create the window
    conn.create_window(
        COPY_DEPTH_FROM_PARENT,   // depth (same as root)
        win,                      // window Id
        screen.root,              // parent window
        0,                        // x
        0,                        // y
        150,                      // width
        150,                      // height
        10,                       // border width
        WindowClass::InputOutput, // class
        screen.root_visual,       // visual
        &Default::default(),
    )?; // masks, not used yet

    // Map the window on the screen
    conn.map_window(win)?;

    // Make sure commands are sent before the sleep, so window is shown
    conn.flush()?;

       let _colormap = screen.default_colormap;
    create_colormap(&conn, ColormapAlloc::All, _colormap, win, screen.root_visual).expect("error creating colormap");

    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}
