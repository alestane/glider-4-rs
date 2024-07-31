#![feature(
    const_option, 
    generic_arg_infer, slice_as_chunks, iter_advance_by,
    map_try_insert, iterator_try_collect
)]

use std::error::Error;

use sdl2::{image::LoadTexture, render::Canvas, surface::Surface, video::Window, EventPump};

mod room {
	use glider::prelude::room;
    pub use room::Id;
	pub const SCREEN_WIDTH:		u32 = room::SCREEN_WIDTH as u32;
	pub const SCREEN_HEIGHT:	u32 = room::SCREEN_HEIGHT as u32;
	pub const VERT_CEILING:		u32 = room::VERT_CEILING as u32;
	pub const VERT_FLOOR:		u32 = room::VERT_FLOOR as u32;
}

use glider::prelude::object;

mod space;
mod resources;
mod game;
mod atlas;
mod draw;
mod test;

use atlas::Atlas;

struct App {
    display: Canvas<Window>,
    sprites: Atlas<Surface<'static>>,
    events: EventPump,
}

fn main() -> Result<(), Box<dyn Error>> {
    let sdl = sdl2::init().unwrap();
    let window = sdl.video().unwrap().window("Glider", room::SCREEN_WIDTH, room::SCREEN_HEIGHT).build().unwrap();
    let display = window.into_canvas().present_vsync().build().unwrap();
    let sprites = {
        let mut sprites = Surface::new(512, 598, display.default_pixel_format())?.into_canvas()?;
        let creator = sprites.texture_creator();
        let pixels = creator.load_texture_bytes(resources::color::SPRITES)?;
        sprites.copy(&pixels, None, None).ok();
        sprites.into_surface()
    };
    let mut app = App {
        display,
        sprites: atlas::glider_sprites(sprites),
        events: sdl.event_pump().unwrap(),
    };
    let mut this_game = app.prepare(&test::stock_house(), &atlas::rooms()).expect("Couldn't load game");
    this_game.play(&mut app).ok();
    Ok(())
}
