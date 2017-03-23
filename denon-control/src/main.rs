// $ printf "MV53\r" | nc -i 1 0005cd221b08.lan 23 | stdbuf -o 0 tr "\r" "\n"
// MV53
// MVMAX 86

#[macro_use]
extern crate conrod;
// extern crate piston_window;
// extern crate image;
// extern crate num;
// extern crate gfx_core;

mod denon_connection;
mod state;
mod operation;
mod parse;

use std::time::Duration;
use std::thread;

use denon_connection::{DenonConnection, State};
use state::PowerState;

use conrod::{Widget, UiCell, Positionable, Sizeable, UiBuilder};
use conrod::widget::{Canvas, Image};
use conrod::image::Map;
use conrod::backend::piston_window::{GlyphCache, convert_event, draw};
// use piston_window::{EventLoop, PistonWindow, UpdateEvent, WindowSettings};
// use piston_window::{ImageSize, G2dTexture, Texture, Size, Window, OpenGL};
// use piston_window::{Event, TextureSettings};
// use image::{ImageBuffer, Pixel};
// use num::complex::Complex;
// use gfx_core::Resources;

#[cfg(test)]
mod test {
    #[test]
    fn bla() {
        assert_eq!(2, 2);
    }
}

// status object shall get the current status of the avr 1912
// easiest way would be a map<Key, Value> where Value is an enum of u32 and String
// Key is derived of a mapping from the protocol strings to constants -> define each string once
// the status object can be shared or the communication thread can be asked about a
// status which queries the receiver if it is not set

fn main() {
    let denon_name = "0005cd221b08.lan";
    let denon_port = 23;

    let dc = DenonConnection::new(denon_name, denon_port);
    let power_status = dc.get(State::power());
    println!("{:?}", power_status);
    if let Ok(State::Power(status)) = power_status {
        if status != PowerState::ON {
            dc.set(State::Power(PowerState::ON)).ok();
            thread::sleep(Duration::from_secs(1));
        }
    }
    println!("current input: {:?}", dc.get(State::source_input()));
    if let Ok(State::MainVolume(current_volume)) = dc.get(State::main_volume()) {
        dc.set(State::MainVolume(current_volume / 2)).ok();
        println!("{:?}", dc.get(State::main_volume()));
        thread::sleep(Duration::from_secs(5));
        dc.set(State::MainVolume(current_volume)).ok();
    }
    thread::sleep(Duration::from_secs(5));
    println!("{:?}", dc.get(State::main_volume()));
    println!("{:?}", dc.get(State::max_volume()));
    dc.stop().ok();
    thread::sleep(Duration::from_secs(5));
}

// =======================================================

fn main2() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_3;

    // Construct the window.
    let mut window: PistonWindow = WindowSettings::new("Canvas Demo", [WIDTH, HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .unwrap();
    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = UiBuilder::new().build();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache = GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // Instantiate the generated list of widget identifiers.
    let ids = &mut Ids::new(ui.widget_id_generator());

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let uicell = ui.set_widgets();
            set_widgets(uicell, ids);
        });

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T {
                    img
                };
                draw(c,
                     g,
                     primitives,
                     &mut text_texture_cache,
                     &image_map,
                     texture_from_image);
            }
        });
    }
}

fn set_widgets(ref mut uicell: UiCell, ids: &mut Ids) {
    // Construct our main `Canvas` tree.
    Canvas::new().set(ids.master, uicell);
}

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    struct Ids {
    // Canvas IDs.
    master,

    // image
    rust_logo,

    image_mouse,
    }
}
