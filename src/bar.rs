use crate::conf::Bar;

use anyhow::Context;
use std::{convert::TryInto, u32};
use x11rb::{
	connection::Connection,
	protocol::xproto::{ConnectionExt, *},
	COPY_DEPTH_FROM_PARENT,
};

struct Rgb {
	r: f64,
	g: f64,
	b: f64,
}

impl Rgb {
	fn new(num: u32) -> Self {
		Rgb {
			r: (num >> 16).into(),
			g: ((num >> 8) & 0xFF).into(),
			b: (num & 0xFF).into(),
		}
	}
}

// https://github.com/psychon/x11rb/issues/328
fn find_xcb_visualtype(
	conn: &xcb::Connection,
	visual_id: u32,
) -> Option<xcb::Visualtype> {
	for root in conn.get_setup().roots() {
		for depth in root.allowed_depths() {
			for visual in depth.visuals() {
				if visual.visual_id() == visual_id {
					return Some(visual);
				}
			}
		}
	}
	None
}

impl Bar {
	pub fn draw(
		&self,
		xcb_conn: &xcb::Connection,
		conn: &(impl Connection + Send + Sync),
		screen: &Screen,
		win: Window,
	) -> anyhow::Result<()> {
		let cairo_conn = unsafe {
			cairo::XCBConnection::from_raw_none(
				xcb_conn.get_raw_conn() as _
			)
		};
		let mut visual =
			find_xcb_visualtype(&xcb_conn, screen.root_visual)
				.with_context(|| {
					"Failed to find visual type of root visual"
				})?;

		let root = screen.root;
		let root_sz = (screen.width_in_pixels, screen.height_in_pixels);

		let x_pos = ((root_sz.0 - self.width) / 2)
			.try_into()
			.with_context(|| "Failed to set X position of bar")?;
		let y_pos = (if self.bottom == Some(true) {
			root_sz.1 - self.height
		} else {
			0
		}) as i16 + self.offset_y.unwrap_or(0); // FIX: use try_into here

		let bg_color = Rgb::new(
			u32::from_str_radix(
				self.background_color.trim_start_matches('#'),
				16,
			)
			.with_context(|| {
				"Failed to convert bar background color to u32"
			})?,
		);

		conn.create_window(
			COPY_DEPTH_FROM_PARENT,         // window depth
			win,                            // window id
			root,                           // parent window
			x_pos,                          // x position
			y_pos,                          // y position
			self.width,                     // width
			self.height,                    // height
			self.border_width.unwrap_or(0), // border width
			WindowClass::InputOutput,       // window class
			screen.root_visual,             // visual
			&Default::default(),            // value list
		)
		.with_context(|| "Failed to create bar window")?;

		let surface = cairo::XCBSurface::create(
			&cairo_conn,
			&cairo::XCBDrawable(win),
			unsafe {
				&cairo::XCBVisualType::from_raw_full(
					&mut visual.base as *mut _
						as *mut cairo_sys::xcb_visualtype_t,
				)
			},
			self.width.into(),
			self.height.into(),
		)
		.with_context(|| "Failed to create cairo surface")?;

		// override default wm decorations
		let values =
			ChangeWindowAttributesAux::default().override_redirect(1);
		conn.change_window_attributes(win, &values).with_context(
			|| "Failed to set bar window attributes to override redirect",
		)?;

		conn.map_window(win).with_context(|| {
			"Failed to map main bar window to root window"
		})?;

		conn.flush()?;

		let ctx = cairo::Context::new(&surface);
		ctx.push_group_with_content(cairo::Content::Color);

		ctx.set_source_rgb(bg_color.r, bg_color.g, bg_color.b);
		ctx.paint();

		ctx.pop_group_to_source();
		ctx.set_operator(cairo::Operator::Source);
		ctx.paint();

		surface.flush();
		xcb_conn
			.flush()
			.then_some(())
			.with_context(|| "Failed to flush xcb connection")?;

		Ok(())
	}
}
