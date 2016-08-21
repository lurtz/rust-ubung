
#[macro_use] extern crate conrod;
extern crate piston_window;
extern crate image;

use conrod::{Canvas, Theme, Widget};
use conrod::{Image, Positionable, Sizeable};
use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};
use piston_window::{ImageSize, G2dTexture, Texture};
use image::ImageBuffer;

fn main() {
    println!("Hello, world!");

    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("Canvas Demo", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).vsync(true).build().unwrap();
    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = conrod::Ui::new(Theme::default());

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    //let image_map = conrod::image::Map::new();
    let image_map = image_map! {
        (RUST_LOGO, rust_logo(&mut window)),
    };


    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| ui.set_widgets(|mut ui| {
          // Construct our main `Canvas` tree.
          Canvas::new().set(MASTER, &mut ui);

          let (w, h) = image_map.get(RUST_LOGO).unwrap().get_size();

          // Instantiate the `Image` at its full size in the middle of the window.
          Image::new().w_h(w as f64, h as f64).middle().set(RUST_LOGO, &mut ui);
        }));

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed(&image_map) {
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     texture_from_image);
            }
        });
    }

}

// Load the Rust logo from our assets folder.
fn rust_logo(window: &mut PistonWindow) -> G2dTexture {
    let factory = &mut window.factory;
    let imbuf = ImageBuffer::new(400,300);
    let settings = piston_window::TextureSettings::new();
    Texture::from_image(factory, &imbuf, &settings).unwrap()
}

// Generate a unique `WidgetId` for each widget.
widget_ids! {

    // Canvas IDs.
    MASTER,

    // image
    RUST_LOGO
}
