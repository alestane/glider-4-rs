#![feature(
    const_option, const_trait_impl, effects, 
    generic_arg_infer, slice_as_chunks, iter_advance_by,
    map_try_insert
)]

use sdl2::{image::LoadTexture, video::Window, render::Canvas, EventPump};

mod room {
	use glider::prelude::room;
	pub const SCREEN_WIDTH:		u32 = room::SCREEN_WIDTH as u32;
	pub const SCREEN_HEIGHT:	u32 = room::SCREEN_HEIGHT as u32;
	pub const VERT_CEILING:		u32 = room::VERT_CEILING as u32;
	pub const VERT_FLOOR:		u32 = room::VERT_FLOOR as u32;
}

mod space;
mod resources;
mod game;
mod atlas;
mod draw;
mod test;

use atlas::Atlas;

struct App<'me> {
    display: Canvas<Window>,
    sprites: Atlas<'me>,
    events: EventPump,
}

fn main() {
    let images = resources::color::assets();
    let sdl = sdl2::init().unwrap();
    let window = sdl.video().unwrap().window("Glider", room::SCREEN_WIDTH, room::SCREEN_HEIGHT).build().unwrap();
    let canvas = window.into_canvas().present_vsync().build().unwrap();
    let loader = canvas.texture_creator();
    let room = loader.load_texture_bytes(images[&200]).unwrap();
    let mut app = App {
        display: canvas,
        sprites: atlas::glider_sprites(loader.load_texture_bytes(images[&128]).unwrap()),
        events: sdl.event_pump().unwrap(),
    };
    game::play(&mut app, &atlas::rooms(&loader), &test::house())
        .ok();
}
