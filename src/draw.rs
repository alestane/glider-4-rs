use std::{collections::HashMap, num::NonZero, iter::repeat};

use glider::{prelude::*, Reference};
use sdl2::{pixels::{Color, PixelFormatEnum}, rect::{Point, Rect}, render::{BlendMode, Canvas, RenderTarget, Texture, TextureCreator}, surface::Surface, video::Window};
use crate::{room::{SCREEN_HEIGHT, SCREEN_WIDTH, VERT_FLOOR}, space, atlas::{self, Atlas}, resources};

fn random() -> u16 {
	use std::sync::LazyLock;
	use random::Source;
	static mut RAND: LazyLock<std::cell::RefCell<random::Default>> = LazyLock::new(|| std::cell::RefCell::new(random::default(
		match std::time::SystemTime::UNIX_EPOCH.elapsed() {
			Ok(length) => length,
			Err(wrong) => wrong.duration(),
		}.as_secs()
	)));
	unsafe { RAND.borrow_mut().read::<u16>() }
}

pub type Frame = Box<dyn Iterator<Item = usize>>;
pub type Animations = HashMap<u8, Frame>;

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

pub trait Visible {
    fn show<Display: Scribe<Builder = TextureCreator<T>>, T, I: Into<Option<usize>>>(&self, display: &mut Display, frame: I) {
        match self.sprite() {
            Some((name, index)) => display.sprite(self.bounds(), name, frame.into().unwrap_or(index)),
            None => self.draw_to(display),
        }
    }
    fn draw_to<Display: Scribe<Builder = TextureCreator<T>>, T>(&self, display: &mut Display);
    fn bounds(&self) -> space::Rect;
    fn sprite(&self) -> Option<(&str, usize)> { None }
}

mod object {
    type Frame = space::Rect;
    pub type Kind = glider::prelude::object::Kind;
    use glider::Transfer;
    use super::*;

    impl Visible for Object {
        fn draw_to<Display, T>(&self, display: &mut Display) 
        where
            Display: Scribe<Builder = TextureCreator<T>>
        {
            type Is = object::Kind;
            match
                match self.kind {
                    Is::Table{width} => {
                        let bounds = space::Rect::from((self.position - (width.get() / 2, 0), Size::from((width, const{ NonZero::new(9).unwrap() }))));
                        draw_table(display, bounds.into())
                    }
                    Is::Shelf{width} => {
                        draw_shelf(display, space::Rect::from((self.position - (width.get() / 2, 0), Size::from((width, const{ NonZero::new(5).unwrap() })))))
                    }
                    Is::Cabinet(size) => {
                        let (width, height) = (size.width() as u32, size.height() as u32);
                        let (left, top) = (self.position.x() as u32 - width / 2, self.position.y() as u32 - height / 2);
                        draw_cabinet(display, Frame::new_unsigned(left, top, left + width, top + height))
                    }
                    Is::Mirror(size) => {
                        let (width, height) = (size.width() as u32, size.height() as u32);
                        let (left, top) = (self.position.x() as u32 - width / 2, self.position.y() as u32 - height / 2);
                        draw_mirror(display, Frame::new_unsigned(left, top, left + width, top + height))
                    }
                    Is::Window(size, ready) => {
                        let (width, height) = (size.width() as u32, size.height() as u32);
                        let (left, top) = (self.position.x() as u32 - width / 2, self.position.y() as u32 - height / 2);
                        draw_window(display, Frame::new_unsigned(left, top, left + width, top + height), ready)
                    }
                    Is::Bonus(..) => Ok(()),
                    what => Err(format!("Unimplemented object: {what:?}")),
                } {
                    Err(e) => eprintln!("{e}"),
                    _ => ()
            }
        }

        fn sprite(&self) -> Option<(&str, usize)> {
            type Is = Kind;
            Some(match self.kind {
                Is::Clock(_) => ("collectible", atlas::CLOCK),
                Is::Battery(..) => ("collectible", atlas::BATTERY),
                Is::Paper(..) => ("collectible", atlas::PAPER),
                Is::FloorVent { .. } => ("blowers", atlas::UP),
                Is::CeilingVent { .. } => ("blowers", atlas::DOWN),
                Is::CeilingDuct { .. } => ("blowers", atlas::DUCT),
                Is::Candle { .. } => ("blowers", atlas::CANDLE),
                Is::Fan { faces: Side::Right, .. } => ("blowers", atlas::FAN_RIGHT),
                Is::Fan { faces: Side::Left, .. } => ("blowers", atlas::FAN_LEFT),
                Is::Switch(Some(..)) => ("power", atlas::TOGGLE),
                Is::Switch(None) => ("power", atlas::SWITCH),
                Is::Outlet{..} => ("power", atlas::OUTLET),
                Is::Macintosh => ("visual", atlas::COMPUTER),
                Is::Books => ("visual", atlas::BOOKS),
                Is::Painting => ("visual", atlas::PAINTING),
                Is::Guitar => ("visual", atlas::GUITAR),
                Is::Stair(direction, ..) => ("stairs", match direction {Vertical::Up => atlas::STAIRS_UP, Vertical::Down => atlas::STAIRS_DOWN}),
                _what => return None
            })
        }
        fn bounds(&self) -> space::Rect {
            let size =  match self.kind {Kind::Table{width, ..} | Kind::Shelf{width, ..} => Size::from((width, const{NonZero::new(9).unwrap() })),
            _ => Size::default()
            };
            let anchor = (Span::Center, Rise::Center);
            space::Rect::from(size / anchor << self.position.as_unsigned())
        }
    }

    fn draw_table<Display, T>(display: &mut Display, bounds: Rect) -> Result<(), String> 
    where
        Display: Scribe + Illuminator<Builder = TextureCreator<T>>
    {
        let bounds = sdl2::rect::Rect::from(bounds);
        let builder = display.get_builder();
        display.outline_rect(bounds, BROWN);
        display.pen(BLACK, &[(bounds.left() + 1, bounds.bottom() - 2), (bounds.right() - 1, bounds.bottom() - 2)])?;
        display.pen(BROWN_LT, &[(bounds.left() + 1, bounds.top() + 1), (bounds.right() - 2, bounds.top() + 1)])?;
        let drop = -(bounds.top().saturating_sub_unsigned(VERT_FLOOR));
        let shadow_bounds = { let mut r = bounds; r.offset(drop / -5, drop); r};
        let shadow = unsafe { shadow::table(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height()))}.or_else(|e| Err(e.to_string()))?;
        display.draw(
            &shadow.as_texture(&builder).map_err(|e| e.to_string())?, 
            None, 
            shadow_bounds
        );

        let center = bounds.center().x();
        let post = Rect::new(center - 2, bounds.bottom(), 5, (VERT_FLOOR - 2).saturating_add_signed(-bounds.bottom()));
        display.fill(BLACK, post)?;
        display.pen(WHITE, &[(center + 1, post.top()), (center + 1, post.bottom())])?;
        display.pen(BROWN_LT, &[(center, post.top()), (center, post.bottom())])?;

        let base_size = unsafe{ Size::from((NonZero::new_unchecked(64), NonZero::new_unchecked(22))) };
        let frame = base_size / (Span::Center, Rise::Center) << Position::from((center as u16, VERT_FLOOR as u16 + 4));
        display.sprite(frame.into(), "visual", atlas::TABLE);

        Ok(())
    }

    fn draw_shelf<Display, T>(display: &mut Display, bounds: space::Rect) -> Result<(), String> 
    where
        Display: Scribe + Illuminator<Builder = TextureCreator<T>>
    {
        let bounds: Rect = bounds.into();
        let builder = display.get_builder();

        let shadow = unsafe { shadow::shelf(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height()))}.or_else(|e| Err(e.to_string()))?;
        display.draw(
            &shadow.as_texture(&builder).map_err(|e| e.to_string())?,
            None, 
            Rect::new(bounds.left() - 15, bounds.top(), bounds.width() + 15, bounds.height() + 15)
        );

        display.fill(BROWN_LT, bounds)?;
        display.pen(BROWN_LT, &[(bounds.left() + 1, bounds.bottom() - 2), (bounds.right() - 1, bounds.bottom() -2)])?;
        display.pen(WHITE, &[(bounds.left() + 1, bounds.top() + 1), (bounds.right() - 2, bounds.top() + 1)])?;

        const FRAME: space::Rect = space::Rect::new_signed(-8, -2, 8, 27);
        
        display.sprite(FRAME << space::Point::from((bounds.left() + 23, bounds.bottom())), "visual", atlas::SHELF);
        display.sprite(FRAME << space::Point::from((bounds.right() - 23, bounds.bottom())), "visual", atlas::SHELF);
        Ok(())
    }

    fn draw_cabinet<Display, T>(display: &mut Display, bounds: Frame) -> Result<(), String> 
    where
        Display: Scribe + Illuminator<Builder = TextureCreator<T>>
    {
        let bounds: Rect = bounds.into();
        let builder = display.get_builder();
        let off = if bounds.bottom() > 280 {
            {
                let bounds = Rect::new(bounds.left(), bounds.top(), bounds.width(), bounds.height() - 5);
                display.outline_rect(bounds, BROWN)?;
            }
            {
                let bounds = Rect::new(bounds.left() + 2, bounds.bottom() - 5, bounds.width() - 5, 5);
                display.fill(BLACK, bounds)?;
            }
            {
                let bounds = Rect::new(bounds.left() - 2, bounds.top(), bounds.width() + 4, 7);
                display.outline_rect(bounds, BROWN_LT)?;
                display.pen(BLACK, &[(bounds.left() + 2, bounds.bottom()), (bounds.right() - 3, bounds.bottom())])?;
            }
            {
                let shadow = shadow::cabinet(unsafe{ NonZero::new_unchecked(bounds.height()) }).map_err(|e| e.to_string())?;
                display.draw(
                    &shadow.as_texture(&builder).map_err(|e| e.to_string())?, 
                    None, 
                    Rect::new(bounds.left() - 15, bounds.top(), 15, bounds.height())
                );
            }
            5u32
        } else {
            {
                let shadow = unsafe { shadow::shelf(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height())) }.map_err(|e| e.to_string())?;
                display.draw(
                    &shadow.as_texture(&builder).map_err(|e| e.to_string())?, 
                    None, 
                    Rect::new(bounds.left() - 15, bounds.top(), bounds.width() + 15, bounds.height() + 15)
                );
            }
            display.outline_rect(bounds, BROWN);
            0
        };
        let panels = bounds.width().max(48) / 48;
        let width = (bounds.width() - (panels + 1) * 5) / panels;
        for h in 0..panels {
            let bounds = Rect::new(bounds.left() + 8 + h as i32 * (width as i32 + 5), bounds.top() + 8i32.saturating_add_unsigned(off), width - 7, bounds.height() - (17 + off * 2));
            display.pen(
                BROWN_LT,
                &[(bounds.left(), bounds.top()), (bounds.left(), bounds.bottom()), (bounds.right(), bounds.bottom())]
            );
            display.pen(
                BLACK,
                &[(bounds.right(), bounds.bottom()), (bounds.right(), bounds.top()), (bounds.left(), bounds.top())]
            );
        }
        Ok(())
    }

    fn draw_mirror<Display, T>(display: &mut Display, bounds: Frame) -> Result<(), String> 
    where
        Display: Scribe + Illuminator<Builder = TextureCreator<T>>
    {
        let outer: Rect = bounds.into();
        let inner = Rect::new(outer.left() + 3, outer.top() + 3, outer.width() - 6, outer.height() - 6);
        display.outline_rect(outer, BROWN)?;
        display.outline_rect(inner, WHITE)?;
        Ok(())
    }

    fn draw_window<Display, T>(display: &mut Display, bounds: Frame, is_open: bool) -> Result<(), String> 
    where
        Display: Scribe + Illuminator<Builder = TextureCreator<T>>
    {
        let bounds: Rect = bounds.into();
        {
            let shadow = unsafe { shadow::window(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height())) }
                .map_err(|e| e.to_string())?;
            let builder = display.get_builder();
            display.draw(&shadow.as_texture(&builder).map_err(|e| e.to_string())?, None, Rect::new(bounds.left() - 10, bounds.top(), bounds.width() + 10, bounds.height() + 5));
        }
        display.limn_rect(bounds, BROWN, BROWN_LT)?;
        display.limn_rect(Rect::new(bounds.left() - 4, bounds.top(), bounds.width() + 8, 6), BROWN, BROWN_LT)?;
        display.limn_rect(Rect::new(bounds.left() - 2, bounds.top() + 6, bounds.width() + 4, 4), BROWN, BROWN_LT)?;
        display.limn_rect(Rect::new(bounds.left() - 4, bounds.bottom() - 6, bounds.width() + 8, 6), BROWN, BROWN_LT)?;
        display.limn_rect(Rect::new(bounds.left() - 2, bounds.bottom() - 10, bounds.width() + 4, 4), BROWN, BROWN_LT)?;
        let casing = Rect::new(bounds.left() + 8, bounds.top() + 16, bounds.width() - 16, bounds.height() - 32);
        display.sink_rect(casing, BROWN)?;
        {
            let pane = Rect::new(casing.left(), casing.top(), casing.width(), casing.height() / 2 + 2);

            display.outline_rect(pane, BROWN)?;
            let pane = Rect::new(pane.left() + 6, pane.top() + 6, pane.width() - 12, pane.height() - 12);
            display.sink_rect(pane, None)?;
            let pane = Rect::new(pane.left() + 2, pane.top() + 2, pane.width() - 4, pane.height() - 4);
            display.sink_rect(pane, None)?;
            let pane = Rect::new(pane.left() + 2, pane.top() + 2, pane.width() - 4, pane.height() - 4);
            display.sink_rect(pane, BLACK)?;
        }
        let height = casing.height() / 2 + 2;
        {
            let pane = Rect::new(casing.left(), casing.top() + height as i32, casing.width(), casing.height() - height);
            display.fill(BLACK, pane);
            let pane = Rect::new(casing.left(), casing.top() + if is_open {26} else {height as i32 - 4}, casing.width(), height);
            display.outline_rect(pane, BROWN)?;
            let pane = Rect::new(pane.left() + 6, pane.top() + 6, pane.width() - 12, pane.height() - 12);
            display.sink_rect(pane, None)?;
            let pane = Rect::new(pane.left() + 2, pane.top() + 2, pane.width() - 4, pane.height() - 4);
            display.sink_rect(pane, None)?;
            let pane = Rect::new(pane.left() + 2, pane.top() + 2, pane.width() - 4, pane.height() - 4);
            display.sink_rect(pane, BLACK)?;
        }
        Ok(())
    }

    mod shadow {
        use super::super::*;
        fn descreen(target: &mut Surface, width: u32) {
            target.without_lock_mut().map(|pixels| {
                for (v, row) in pixels.as_chunks_mut::<4>().0.chunks_mut(width as usize).enumerate() {
                    for pair in row[(v % 2)..].chunks_mut(2) {
                        pair[0] = [0x00; 4];
                    }
                }
            });
        }
        
        pub fn table(width: NonZero<u32>, height: NonZero<u32>) -> Result<Surface<'static>, Box<dyn std::error::Error>> {
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
        
            descreen(&mut shadow, width);
            Ok(shadow)
        }
        
        pub fn shelf(width: NonZero<u32>, height: NonZero<u32>) -> Result<Surface<'static>, Box<dyn std::error::Error>> {
            let (width, height) = (width.get(), height.get());
            let mut shadow = {
                let mut canvas = Surface::new(width + 15, height + 15, PixelFormatEnum::RGBA8888)?.into_canvas()?;
                canvas.set_draw_color((0, 0, 0, 0));
                canvas.clear();
                canvas.set_draw_color(BLACK);
                for left in 0..15 {
                    canvas.fill_rect(Rect::new(left, 15 - left, width, height))?;
                }
                canvas.into_surface()
            };
            descreen(&mut shadow, width + 15);
            Ok(shadow)
        }
        
        pub fn cabinet(height: NonZero<u32>) -> Result<Surface<'static>, Box<dyn std::error::Error>> {
            let height = height.get();
            let mut shadow = {
                let mut canvas = Surface::new(15, height, PixelFormatEnum::RGBA8888)?.into_canvas()?;
                canvas.set_draw_color((0, 0, 0, 0));
                canvas.clear();
                canvas.set_draw_color(BLACK);
                for h in 0..15 {
                    canvas.draw_line((h, 20 - h), (h, (height as i32 - 10) + (h * 10) / 15))?;
                }
                canvas.into_surface()
            };
            descreen(&mut shadow, 15);
            Ok(shadow)
        }
        
        pub fn window(width: NonZero<u32>, height: NonZero<u32>) -> Result<Surface<'static>, Box<dyn std::error::Error>> {
            let (width, height) = (width.get(), height.get());
            let mut shadow = {
                let mut canvas = Surface::new(width + 10, height + 5, PixelFormatEnum::RGBA8888)?.into_canvas()?;
                canvas.set_draw_color((0, 0, 0, 0));
                canvas.clear();
                canvas.set_draw_color(BLACK);
                for left in 0..5 {
                    canvas.fill_rect(Rect::new(left + 5, 5 - left, width, height))?;
                    canvas.draw_line((left, 10 - left), (left, 15 + left))?;
                    canvas.draw_line((left, height as i32 - (5 + left)), (left, height as i32 + left))?;
                }
                canvas.into_surface()
            };
            descreen(&mut shadow, width + 10);
            Ok(shadow)
        
        }

    }    
}    

mod hazard {
    
}

trait Illuminator {
    type Builder;
    fn get_builder(&self) -> Self::Builder;
}

impl Illuminator for Canvas<Window> {
    type Builder = sdl2::render::TextureCreator<sdl2::video::WindowContext>;
    fn get_builder(&self) -> Self::Builder {
        self.texture_creator()
    }
}

impl<T> Illuminator for (&mut Canvas<Window>, T) {
    type Builder = <Canvas<Window> as Illuminator>::Builder;
    fn get_builder(&self) -> Self::Builder { self.0.get_builder() }
}

pub trait Scribe : Illuminator {
    fn draw(&mut self, pixels: &Texture, source: impl Into<Option<Rect>>, dest: impl Into<Option<Rect>>);
    fn pen<const N: usize>(&mut self, stroke: impl Into<Color>, vertices: &[(i32, i32); N]) -> Result<(), String>;
    fn fill(&mut self, tone: impl Into<Color>, bounds: Rect) -> Result<(), String>;
    fn outline_rect(&mut self, bounds: Rect, fill: impl Into<Color>) -> Result<(), String>;
    fn limn_rect(&mut self, bounds: Rect, fill: impl Into<Color>, hilite: impl Into<Color>) -> Result<(), String>;
    fn sink_rect(&mut self, bounds: Rect, fill: impl Into<Option<Color>>) -> Result<(), String>;
    fn draw_wall(&mut self, theme: &Texture, tiles: &[u8]);
    fn sprite(&mut self, bounds: space::Rect, name: &str, index: usize);
    fn draw_room(&mut self, play: &glider::Play, times: &mut HashMap<u8, Box<dyn Iterator<Item = usize>>>, sprites: &Atlas, backdrop: &Texture);
}

impl<R:RenderTarget, T> Scribe for (&mut Canvas<R>, &Atlas<'_>) where Self: Illuminator<Builder = sdl2::render::TextureCreator<T>> {
    fn draw(&mut self, pixels: &Texture, source: impl Into<Option<Rect>>, dest: impl Into<Option<Rect>>) {
        self.0.copy(pixels, source, dest)
            .expect("failed to draw to canvas");
    }

    fn pen<const N: usize>(&mut self, stroke: impl Into<Color>, vertices: &[(i32, i32); N]) -> Result<(), String> {
        let display = &mut *self.0;
        display.set_draw_color(stroke.into());
        display.draw_lines(vertices.map(|(x, y)| sdl2::rect::Point::new(x, y) ).as_ref())
    }

    fn fill(&mut self, tone: impl Into<Color>, bounds: Rect) -> Result<(), String> {
        let display = &mut *self.0;
        display.set_draw_color(tone.into());
        display.fill_rect(bounds)
    }

    fn outline_rect(&mut self, bounds: Rect, fill: impl Into<Color>) -> Result<(), String> {
        let display = &mut *self.0;
        display.set_draw_color(fill.into());
        display.fill_rect(bounds)?;
        // display.set_blend_mode(BlendMode::Blend);
        display.set_draw_color(BLACK);
        display.draw_rect(bounds)
    }

    fn limn_rect(&mut self, bounds: Rect, fill: impl Into<Color>, hilite: impl Into<Color>) -> Result<(), String> {
        self.outline_rect(bounds, fill)?;
        let display = &mut *self.0;
        display.set_draw_color(hilite);
        display.draw_lines([
            Point::new(bounds.left() + 1, bounds.top() +1),
            Point::new(bounds.right() - 2, bounds.top() + 1),
            Point::new(bounds.right() - 2, bounds.bottom() - 2),
        ].as_ref())
    }

    fn sink_rect(&mut self, bounds: Rect, fill: impl Into<Option<Color>>) -> Result<(), String> {
        if let Some(fill) = fill.into() {
            self.outline_rect(bounds, fill)?;
        }
        let display = &mut *self.0;
        display.set_draw_color(BROWN_LT);
        display.draw_lines([
            Point::new(bounds.left() - 1, bounds.top()),
            Point::new(bounds.left() - 1, bounds.bottom()),
            Point::new(bounds.right() - 1, bounds.bottom()),
        ].as_ref())?;
        display.set_draw_color(GRAY_DK);
        display.draw_lines([
            Point::new(bounds.left(), bounds.top() - 1),
            Point::new(bounds.right(), bounds.top() - 1),
            Point::new(bounds.right(), bounds.bottom()),
        ].as_ref())
    }

    fn draw_wall(&mut self, theme: &Texture, tiles: &[u8]) {
        const TILE_WIDTH: u32 = SCREEN_WIDTH / 8;
        for (index, &slice) in tiles.iter().enumerate() {
            self.draw(&theme,
                Rect::new(slice as i32 * TILE_WIDTH as i32, 0, TILE_WIDTH, SCREEN_HEIGHT),
                Rect::new(index as i32 * TILE_WIDTH as i32, 0, TILE_WIDTH, SCREEN_HEIGHT)
            );
        }
    }

    fn sprite(&mut self, bounds: space::Rect, name: &str, index: usize) {
        let (wedge, tex) = self.1.get(name);
        self.draw(tex, wedge[index], bounds);
    }

    fn draw_room(&mut self, play: &glider::Play, times: &mut Animations, sprites: &Atlas, backdrop: &Texture) {
        fn advance(lookup: &mut Animations, id: u8) -> Option<usize> {
            let index = lookup.get_mut(&id).and_then(|a| a.next());
            if index.is_none() {
                lookup.remove(&id);
            }
            index
        }

        let display = &mut *self.0;
        display.set_draw_color(Color::RGB(0, 0, 0));
        display.clear();
        let (mut player_position, facing, backward) = play.player();
        let (slides, pixels) = sprites.get(match facing {Side::Left => "glider.left", Side::Right => "glider.right"} );
        let frame: sdl2::rect::Rect = slides[advance(times, 0).unwrap_or(if backward {atlas::TIPPED} else {atlas::LEVEL})].into();
        if frame.height() > 20 {player_position.1 -= frame.height() as i16 / 2 - 10};
        if play.dark() {
            display.set_draw_color(BLACK);
            display.clear();
            for item in play.active_items().filter(|&o| matches!(o.kind, object::Kind::Switch(None))) {
                item.show(self, 0);
            }
        } else {
            self.draw(&backdrop, None, None);
            for item in play.active_items().filter(|&o| matches!(o.kind, object::Kind::Mirror(..))) {
                let bounds: Rect = space::Rect::default().into();
                self.0.set_clip_rect(Rect::new(bounds.left() + 3, bounds.top() + 3, bounds.width() - 6, bounds.height() - 6));
                self.draw(pixels, frame, frame.centered_on((player_position.0 as i32 - 16, player_position.1 as i32  - 32)));
            }
            self.0.set_clip_rect(None);
            for item in play.active_items().filter(|&o| o.dynamic()) {
                item.show(self, 0);
            }
        }
        for (id, hazard, position, is_on) in play.active_hazards() {
        	let position: space::Point = position.into();
            let (width, height, group, range) = match hazard {
                Enemy::Dart => (64, 22, "dart", atlas::FLYING),
            	Enemy::Balloon => (32, 32, "balloon", atlas::RISING),
                Enemy::Copter => (32, 32, "copter", atlas::FALLING),
                Enemy::Flame => (11, 12, "fire", atlas::FLAME),
                Enemy::Shock => (32, 25, "power", atlas::SPARK),
                _ => continue
            };
            if !is_on { continue; }
            let frame = if let Some(frame) = times.get_mut(&id).and_then(|seq| seq.next()) {
				frame
			} else {
				let skip = id as usize % (range.end - range.start);
				let mut c = range.cycle().map(|i| repeat(i).take(2)).flatten();
				c.advance_by(skip).ok();
				let frame = unsafe{ c.next().unwrap_unchecked() };
				times.insert(id, Box::new(c));
				frame
			};
			self.sprite(Rect::from_center(position, width, height).into(), group, frame);
        }
        self.draw(pixels, frame, frame.centered_on((player_position.0 as i32, player_position.1 as i32)));
        let frame: sdl2::rect::Rect = slides[atlas::SHADOW].into();
        self.draw(pixels, frame, frame.centered_on((player_position.0 as i32, (VERT_FLOOR + frame.height() / 2) as i32)));
        self.0.present();
    }
}
