
#[macro_use]
extern crate conrod;
extern crate piston_window;
extern crate image;
extern crate num;

use conrod::{Widget, widget};
use conrod::{Positionable, Sizeable};
use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};
use piston_window::{ImageSize, G2dTexture, Texture, Size};
use image::{ImageBuffer, Pixel};
use num::complex::Complex;

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
    let mut ui = conrod::UiBuilder::new().build();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // Instantiate the generated list of widget identifiers.
    let ids = &mut Ids::new(ui.widget_id_generator());

    // The image map describing each of our widget->image mappings (in our case, none).
    //let image_map = conrod::image::Map::new();
    let image_map = image_map! {
        (ids.rust_logo, rust_logo(&mut window)),
    };


    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }


        event.update(|_| {
          let ref mut uicell = ui.set_widgets();
          // Construct our main `Canvas` tree.
          widget::Canvas::new().set(ids.master, uicell);

          let (w, h) = image_map.get(&ids.rust_logo).unwrap().get_size();

          // Instantiate the `Image` at its full size in the middle of the window.
          widget::Image::new().w_h(w as f64, h as f64).middle().set(ids.rust_logo, uicell);
        });

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     &image_map,
                                                     texture_from_image);
            }
        });
    }

}

// Load the Rust logo from our assets folder.
fn rust_logo(window: &mut PistonWindow) -> G2dTexture {
    use piston_window::Window;

    let size = window.draw_size();
    let factory = &mut window.factory;
    let px_func = |x: u32, y: u32| px_func(x, y, size);
    let imbuf = ImageBuffer::from_fn(size.width, size.height, px_func);
    let settings = piston_window::TextureSettings::new();
    Texture::from_image(factory, &imbuf, &settings).unwrap()
}

fn px_func(x: u32, y: u32, size: Size) -> image::Rgba<u8> {
    let iterations = 20;
    let mut i: u32 = 0;
    let cx = (x as f32 - size.width as f32/2.0) / (size.width as f32 / 4.0);
    let cy = (y as f32 - size.height as f32/2.0) / (size.height as f32 / 4.0);
    let c = Complex::new(cx, cy);
    let mut z = c;
    while i < iterations && z.norm() < 1000.0 {
        z = z*z + c;
        i = i+1;
    }
    let (r,g,b,a) = taken_iterations_to_rgba(i, iterations);
    image::Rgba::<u8>::from_channels(r, g, b, a)
}

fn taken_iterations_to_rgba(i: u32, max_iterations: u32) -> (u8, u8, u8, u8) {
    // i > max_iterations                                -> (  0,   0, 0, 255)
    // i < max_iterations && i >= max_iterations*2/3     -> (  r,   0, 0, 255)
    // i < max_iterations*2/3 && i >= max_iterations*1/3 -> (255,   g, 0, 255)
    // i < max_iterations*1/3                            -> (255, 255, b, 255)
    let red_border = max_iterations * 2 / 3;
    let green_border = max_iterations * 1 / 3;

    let mut r = 0;
    if i < max_iterations && i > red_border {
        let upper_bound = max_iterations - red_border;
        let reduced_i = i - red_border;
        let inv_r = ((reduced_i as f32 / upper_bound as f32) * 254.0) as u8;
        r = 255 - inv_r;
    }
    if i <= red_border {
        r = 255;
    }

    let mut g = 0;
    if i < red_border && i >= green_border {
        let upper_bound = red_border - green_border;
        let reduced_i = i - green_border;
        let inv_g = ((reduced_i as f32 / upper_bound as f32) * 254.0) as u8;
        g = 255 - inv_g;
    }
    if i <= green_border {
        g = 255;
    }

    let mut b = 0;
    if i < green_border {
        let inv_b = ((i as f32 / green_border as f32) * 254.0) as u8;
        b = 255 - inv_b;
    }

    let a = 255;
    (r,g,b,a)
}

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    struct Ids {
    // Canvas IDs.
    master,

    // image
    rust_logo,
    }
}

