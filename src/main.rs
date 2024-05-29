#![feature(const_trait_impl, effects, generic_arg_infer, slice_as_chunks)]

use sdl2::{image::LoadTexture, video::Window, render::Canvas, EventPump};

const SCREEN_HEIGHT: u32 = 342;
const SCREEN_WIDTH: u32 = 512;
const VERT_CEILING: u32 = 24;
const VERT_FLOOR: u32 = 325;

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
    let window = sdl.video().unwrap().window("Glider", SCREEN_WIDTH, SCREEN_HEIGHT).build().unwrap();
    let canvas = window.into_canvas().present_vsync().build().unwrap();
    let loader = canvas.texture_creator();
    let room = loader.load_texture_bytes(images[&200]).unwrap();
    let mut app = App {
        display: canvas,
        sprites: atlas::glider_sprites(loader.load_texture_bytes(images[&128]).unwrap()),
        events: sdl.event_pump().unwrap(),
    };
    game::run(&mut app, room, &test::new());
}
