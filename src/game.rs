use sdl2::{pixels::Color, render::{Canvas, Texture}, video::Window};
use glider::{Room, ObjectKind, Side, Entrance, Object};
use crate::{atlas, draw, SCREEN_HEIGHT, SCREEN_WIDTH};

fn appearance(kind: &ObjectKind) -> Option<(&'static str, usize)> {
    Some(match kind {
        ObjectKind::Clock(_) => ("collectible", atlas::CLOCK),
        ObjectKind::FloorVent { .. } => ("blowers", atlas::UP),
        ObjectKind::Macintosh => ("visual", atlas::COMPUTER),
        what => { eprintln!("no sprite: {what:?}"); return None }
    })
}

fn show(display: &mut Canvas<Window>, object: &Object, atlas: &atlas::Atlas) {
    match appearance(&object.object_is) {
        Some((name, index)) => {
            let (wedge, tex) = atlas.get(name);
            let _ = display.copy(tex, Some(wedge[index].into()), Some(crate::space::Rect::from(object.bounds).into()));
        }
        None => {
            draw::thing(display, object, &atlas);
        }
    }
}

pub fn run(context: &mut crate::App, theme: Texture, room: &Room) {
    let display = &mut context.display;
    let loader = display.texture_creator();

    let mut backdrop = loader.create_texture_target(None, SCREEN_WIDTH, SCREEN_HEIGHT).expect("Failed to create backdrop texture");
    let _ = display.with_texture_canvas(&mut backdrop, 
        |display| {
            draw::wall(display, &theme, &room.tile_order);
            for object in room.objects.iter().filter(|&object| !object.dynamic()) {
                show(display, object, &context.sprites);
            }
        }
    );
    let play = room.start(Entrance::Flying(Side::Left), true, true);
    'game: loop {
        for event in context.events.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit{..} => break 'game,
                _ => ()
            }
        }
        display.set_draw_color(Color::RGB(0, 0, 0));
        display.clear();
        let _ = display.copy(&backdrop, None, None);
        for item in play.active_items().filter(|&o| o.dynamic()) {
            show(display, item, &context.sprites);
        }
        let (position, facing) = play.player();
        let (slides, pixels) = context.sprites.get(match facing {Side::Left => "glider.left", Side::Right => "glider.right"} );
        let frame: sdl2::rect::Rect = slides[atlas::LEVEL].into();
        display.copy(pixels, frame, frame.centered_on((position.0 as i32, position.1 as i32))).ok();
        let frame: sdl2::rect::Rect = slides[atlas::SHADOW].into();
        display.copy(pixels, frame, frame.centered_on((position.0 as i32, crate::VERT_FLOOR as i32))).ok();
        display.present();
    }
}