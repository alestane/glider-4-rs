use std::num::NonZero;

use glider::{Object, ObjectKind};
use sdl2::{pixels::{Color, PixelFormatEnum}, rect::Rect, render::{BlendMode, Canvas, Texture}, surface::Surface, video::Window};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, VERT_FLOOR, space, atlas, resources};

const BLACK     : Color = Color::RGB(0x00, 0x00, 0x00);
const WHITE     : Color = Color::RGB(0xFF, 0xFF, 0xFF);
const YELLOW    : Color = Color::RGB(0xFF, 0xFF, 0x00);
const VIOLET    : Color = Color::RGB(0xFF, 0x00, 0xFF);
const RED       : Color = Color::RGB(0xFF, 0x00, 0x00);
const BLUE_LT   : Color = Color::RGB(0x00, 0xFF, 0xFF);
const BROWN     : Color = Color::RGB(0x56, 0x2D, 0x05);
const BROWN_LT  : Color = Color::RGB(0x91, 0x71, 0x3A);
const GRAY_DK   : Color = Color::RGB(0x80, 0x80, 0x80);
const BLUE      : Color = Color::RGB(0x00, 0x00, 0xFF);
const GREEN_LT  : Color = Color::RGB(0x1F, 0xB8, 0x14);

pub fn wall(display: &mut Canvas<Window>, theme: &Texture<'_>, tiles: &[u16]) {
    const TILE_WIDTH: u32 = SCREEN_WIDTH / 8;
    for (index, &slice) in tiles.iter().enumerate() {
        let _ = display.copy(&theme, 
            Rect::new(slice as i32 * TILE_WIDTH as i32, 0, TILE_WIDTH, SCREEN_HEIGHT), 
            Rect::new(index as i32 * TILE_WIDTH as i32, 0, TILE_WIDTH, SCREEN_HEIGHT)
        );
    }
}

fn table_shadow(width: NonZero<u32>, height: NonZero<u32>) -> Result<Surface<'static>, Box<dyn std::error::Error>> {
    let (width, height) = (width.get(), height.get());
    let mut shadow = {
        let mut canvas = Surface::new(width, height, PixelFormatEnum::RGBA8888)?.into_canvas()?;
        let builder = canvas.texture_creator();
        let mut circle = builder.create_texture_static(PixelFormatEnum::ABGR8888, 256, 256)?;
        circle.update(None, resources::CIRCLE, 256 * 4)?;
        circle.set_blend_mode(BlendMode::Blend);
        canvas.copy(&circle, None, None)?;
        canvas.into_surface()
    };

    shadow.without_lock_mut().map(|pixels| {
        for (v, row) in pixels.as_chunks_mut::<4>().0.chunks_mut(width as usize).enumerate() {
            for pair in row[(v % 2)..].chunks_mut(2) {
                    pair[0] = [0x00; 4];
            }
        }
    }); 
    Ok(shadow)
}

pub fn table(display: &mut Canvas<Window>, bounds: space::Rect, base: &Texture<'_>, frame: space::Rect) -> Result<(), String>{
    let bounds = sdl2::rect::Rect::from(bounds);
    display.set_draw_color(BROWN);
    display.fill_rect(bounds)?;
    display.set_draw_color(BLACK);
    display.draw_rect(bounds)?;
    display.draw_line((bounds.left() + 1, bounds.bottom() - 2), (bounds.right() - 1, bounds.bottom() - 2))?;
    display.set_draw_color(BROWN_LT);
    display.draw_line((bounds.left() + 1, bounds.top() + 1), (bounds.right() - 2, bounds.top() + 1))?;
    let builder = display.texture_creator();
    //let shadow = unsafe { table_shadow(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height())) }
    //    .map_err(|e| e.to_string())?;
    let drop = -(bounds.top().saturating_sub_unsigned(VERT_FLOOR));
    let shadow_bounds = { let mut r = bounds; r.offset(drop / -5, drop); r};
    let shadow = unsafe { table_shadow(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height()))}.or_else(|e| Err(e.to_string()))?;
    display.copy(&shadow.as_texture(&builder).map_err(|e| e.to_string())?, None, shadow_bounds)?;

    let center = bounds.center().x();
    let post = Rect::new(center - 2, bounds.bottom(), 5, (VERT_FLOOR - 2).saturating_add_signed(-bounds.bottom()));
    display.set_draw_color(BLACK);
    display.fill_rect(post)?;
    display.set_draw_color(WHITE);
    display.draw_line((center + 1, post.top()), (center + 1, post.bottom()))?;
    display.set_draw_color(BROWN_LT);
    display.draw_line((center, post.top()), (center, post.bottom()))?;

    let frame: Rect = frame.into();
    display.copy(base, frame, frame.centered_on((center, VERT_FLOOR as i32 + 4)))?;

    Ok(())
}

pub fn thing(display: &mut Canvas<Window>, object: &Object, decor: &atlas::Atlas) {
    match 
        match object.object_is {
            ObjectKind::Table => {
                let (frame, pixels) = decor.get("visual");
                table(display, object.bounds.into(), pixels, frame[atlas::TABLE])
            }
            _ => Ok(())
        } {
            Err(e) => eprintln!("{e}"),
            _ => ()
    }

}