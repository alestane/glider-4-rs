use std::{collections::HashMap, iter::repeat, num::NonZero, cell::RefCell};

use glider::{prelude::*, Bounds, Reference};
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
pub type Animations = RefCell<HashMap<usize, Frame>>;

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
    fn show<Display: Scribe>(&self, display: &mut Display);
}

trait Animator {
    fn check(&self, id: usize) -> Option<usize>;
}

impl Animator for Animations {
    fn check(&self, id: usize) -> Option<usize> { 
        let mut list = self.borrow_mut();
        let index = list.get_mut(&id).and_then(|id| id.next());
        if index.is_none() { list.remove(&id); }
        index
     }
}

mod object {
    type Frame = space::Rect;
    pub type Kind = glider::prelude::object::Kind;
    pub type Motion = glider::prelude::object::Motion;
    use std::ops::Range;

    use super::*;

    impl Visible for Object {
        fn show<Display: Scribe>(&self, display: &mut Display) {
            type Is = object::Kind;
            match
                match self.kind {
                    Is::Table{width} => {
                        let bounds = Size::from((width, const{ NonZero::new(9).unwrap() })) / (Span::Center, Rise::Top) << *self.position;
                        draw_table(display, Frame::from(bounds).into())
                    }
                    Is::Shelf{width} => {
                        draw_shelf(display, space::Rect::from((self.position - (width.get() as i16 / 2, 0), Size::from((width, const{ NonZero::new(5).unwrap() })))))
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
                    Is::Bonus(..) | Is::Exit{..} => Ok(()),
                    _ => return (None, self).show(display),
                } {
                    Err(e) => eprintln!("{e}"),
                    _ => ()
            }
        }
    
    }

    impl<I: Into<Option<usize>> + Copy> Visible for (I, &Object) {
        fn show<Display: Scribe>(&self, display: &mut Display) {
            type Is = Kind;
            let (category, frame, anchor) = match self.1.kind {
                Is::Table{..} | Is::Shelf{..} | Is::Cabinet(..) | Is::Window{..} | Is::Mirror(..) 
                    => return self.1.show(display),
                Is::Clock(_) => ("collectible", atlas::CLOCK, BOTTOM),
                Is::Battery(..) => ("collectible", atlas::BATTERY, BOTTOM),
                Is::Paper(..) => ("collectible", atlas::PAPER, BOTTOM),
                Is::RubberBands(..) => ("collectible", atlas::BANDS, BOTTOM),
                Is::Grease{ready: true, ..} => ("grease", atlas::UPRIGHT, BOTTOMRIGHT),
                Is::Grease{ready: false, progress: Range{start: ..-1, ..}} => ("grease", atlas::TIPPING, BOTTOMRIGHT),
                Is::Grease{progress: Range{start, ..}, ..} => {
                    if start > 0 {
                        display.fill(BLACK, Rect::new(self.1.position.x() as i32, self.1.position.y() as i32, start as u32, 2)).ok();
                    }
                    ("grease", atlas::TIPPED, BOTTOMRIGHT)
                }
                Is::FloorVent { .. } => ("blowers", atlas::UP, TOP),
                Is::CeilingVent { .. } => ("blowers", atlas::DOWN, BOTTOM),
                Is::CeilingDuct { .. } => ("blowers", atlas::DUCT, BOTTOM),
                Is::Candle { .. } => ("blowers", atlas::CANDLE, BOTTOM),
                Is::Fan { faces: Side::Right, .. } => ("blowers", atlas::FAN_RIGHT, BOTTOM),
                Is::Fan { faces: Side::Left, .. } => ("blowers", atlas::FAN_LEFT, BOTTOM),
                Is::Switch(..) => ("power", atlas::TOGGLE, CENTER),
                Is::Lights => ("power", atlas::SWITCH, CENTER),
                Is::Thermostat => ("power", atlas::THERMO, CENTER),
                Is::Outlet{progress: Range{start: phase@..=0, ..}} => ("shock", phase.rem_euclid(2) as usize, CENTER),
                Is::Outlet{..} => ("power", atlas::OUTLET, CENTER),
                Is::Shredder{..} => ("visual", atlas::SHREDDER, CENTER),
                Is::Drip {..} => ("water", atlas::STILL_DRIP, TOP),
                Is::Drop(Motion{limit: Range{start, ..}, ..}) => ("water", 4usize.saturating_add_signed(start as isize / 2).min(4), TOP),
                Is::Macintosh => ("visual", atlas::COMPUTER, BOTTOM),
                Is::Books => ("visual", atlas::BOOKS, BOTTOM),
                Is::Basket => ("visual", atlas::BASKET, BOTTOM),
                Is::Painting => ("visual", atlas::PAINTING, CENTER),
                Is::Guitar => ("visual", atlas::GUITAR, BOTTOM),
                Is::Flame => ("fire", 0, CENTER),
                Is::Ball(..) => ("ball", atlas::BALL, CENTER),
                Is::Teakettle{..} => ("blowers", atlas::KETTLE, BOTTOM),
                Is::Fishbowl{..} => ("fish", atlas::FISHBOWL, BOTTOM),
                Is::Fish(Motion{limit: Range{start: 1.., ..}, ..}) => ("fish", 4, CENTER),
                Is::Fish(Motion{velocity, ..}) => ("fish", match velocity {..-15 => 1, -15..=15 => 2, 16.. => 3}, CENTER),
                Is::Toaster{..} => ("toaster", 0, BOTTOM),
                Is::Toast(_, slot) => return {display.clipping(Rect::new(self.1.position.x() as i32 - 16, 0, 32, slot as u32), 
                    |display| display.sprite(self.1.position.into(), CENTER, "toaster", self.0.into().unwrap_or(1))
                );},
                Is::Balloon(..) => ("balloon", atlas::POPPED, CENTER),
                Is::Copter(..) => ("copter", atlas::CRUMPLED, CENTER),
                Is::Dart(..) => ("dart", atlas::CRUSHED, CENTER),
                Is::Stair(direction, ..) => ("stairs", match direction {Vertical::Up => atlas::STAIRS_UP, Vertical::Down => atlas::STAIRS_DOWN}, BOTTOM),
                #[cfg(debug_assertions)]
                _ => return eprintln!("Object {:?} NOT IMPLEMENTED yet.", self.1)
            };
            display.sprite(space::Point::from(self.1.position).into(), anchor, category, self.0.into().unwrap_or(frame));
        }
    }

    fn draw_table<Display: Scribe>(display: &mut Display, bounds: Rect) -> Result<(), String> {
        let bounds = sdl2::rect::Rect::from(bounds);
        let builder = display.get_builder();
        display.outline_rect(bounds, BROWN)?;
        display.pen(BLACK, &[(bounds.left() + 1, bounds.bottom() - 2), (bounds.right() - 1, bounds.bottom() - 2)])?;
        display.pen(BROWN_LT, &[(bounds.left() + 1, bounds.top() + 1), (bounds.right() - 2, bounds.top() + 1)])?;
        let drop = -(bounds.top().saturating_sub_unsigned(VERT_FLOOR));
        let shadow_bounds = { let mut r = bounds; r.offset(drop / -5, drop); r};
        let shadow = unsafe { shadow::table(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height()))}.or_else(|e| Err(e.to_string()))?;
        display.draw(
            &builder.load(&shadow)?, 
            None, 
            shadow_bounds
        );

        let center = bounds.center().x();
        let post = Rect::new(center - 2, bounds.bottom(), 5, (VERT_FLOOR - 2).saturating_add_signed(-bounds.bottom()));
        display.fill(BLACK, post)?;
        display.pen(WHITE, &[(center + 1, post.top()), (center + 1, post.bottom())])?;
        display.pen(BROWN_LT, &[(center, post.top()), (center, post.bottom())])?;

        display.sprite((center as i16, VERT_FLOOR as i16 + 4), CENTER, "visual", atlas::TABLE);

        Ok(())
    }

    fn draw_shelf<Display: Scribe>(display: &mut Display, bounds: Frame) -> Result<(), String> {
        let bounds: Rect = bounds.into();
        let builder = display.get_builder();

        let shadow = unsafe { shadow::shelf(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height()))}.or_else(|e| Err(e.to_string()))?;
        display.draw(
            &builder.load(&shadow)?,
            None, 
            Rect::new(bounds.left() - 15, bounds.top(), bounds.width() + 15, bounds.height() + 15)
        );

        display.fill(BROWN_LT, bounds)?;
        display.pen(BROWN_LT, &[(bounds.left() + 1, bounds.bottom() - 2), (bounds.right() - 1, bounds.bottom() -2)])?;
        display.pen(WHITE, &[(bounds.left() + 1, bounds.top() + 1), (bounds.right() - 2, bounds.top() + 1)])?;
        
        display.sprite((bounds.left() as i16 + 23, bounds.bottom() as i16 - 2), TOP, "visual", atlas::SHELF);
        display.sprite((bounds.right() as i16 - 23, bounds.bottom() as i16 - 2), TOP, "visual", atlas::SHELF);
        Ok(())
    }

    fn draw_cabinet<Display: Scribe>(display: &mut Display, bounds: Frame) -> Result<(), String> 
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
                    &builder.load(&shadow)?, 
                    None, 
                    Rect::new(bounds.left() - 15, bounds.top(), 15, bounds.height())
                );
            }
            5u32
        } else {
            {
                let shadow = unsafe { shadow::shelf(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height())) }.map_err(|e| e.to_string())?;
                display.draw(
                    &builder.load(&shadow)?, 
                    None, 
                    Rect::new(bounds.left() - 15, bounds.top(), bounds.width() + 15, bounds.height() + 15)
                );
            }
            display.outline_rect(bounds, BROWN)?;
            0
        };
        let panels = bounds.width().max(48) / 48;
        let width = (bounds.width() - (panels + 1) * 5) / panels;
        for h in 0..panels {
            let bounds = Rect::new(bounds.left() + 8 + h as i32 * (width as i32 + 5), bounds.top() + 8i32.saturating_add_unsigned(off), width - 7, bounds.height() - (17 + off * 2));
            display.pen(
                BROWN_LT,
                &[(bounds.left(), bounds.top()), (bounds.left(), bounds.bottom()), (bounds.right(), bounds.bottom())]
            )?;
            display.pen(
                BLACK,
                &[(bounds.right(), bounds.bottom()), (bounds.right(), bounds.top()), (bounds.left(), bounds.top())]
            )?;
        }
        Ok(())
    }

    fn draw_mirror<Display: Scribe>(display: &mut Display, bounds: Frame) -> Result<(), String> {
        let outer: Rect = bounds.into();
        let inner = Rect::new(outer.left() + 3, outer.top() + 3, outer.width() - 6, outer.height() - 6);
        display.outline_rect(outer, BROWN)?;
        display.outline_rect(inner, WHITE)?;
        Ok(())
    }

    fn draw_window<Display: Scribe>(display: &mut Display, bounds: Frame, is_open: bool) -> Result<(), String> {
        let bounds: Rect = bounds.into();
        {
            let shadow = unsafe { shadow::window(NonZero::new_unchecked(bounds.width()), NonZero::new_unchecked(bounds.height())) }
                .map_err(|e| e.to_string())?;
            let builder = display.get_builder();
            display.draw(&builder.load(&shadow)?, None, Rect::new(bounds.left() - 10, bounds.top(), bounds.width() + 10, bounds.height() + 5));
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
            display.fill(BLACK, pane)?;
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

mod room {
    use std::ops::Range;

    use super::*; 

    impl Visible for (&Texture<'_>, [u8; 8]) {
        fn show<Display: Scribe>(&self, display: &mut Display) {
            const TILE_WIDTH: u32 = SCREEN_WIDTH / 8;
            let &(theme, tiles) = self;
            for (index, &slice) in tiles.iter().enumerate() {
                display.draw(&theme,
                    Rect::new(slice as i32 * TILE_WIDTH as i32, 0, TILE_WIDTH, SCREEN_HEIGHT),
                    Rect::new(index as i32 * TILE_WIDTH as i32, 0, TILE_WIDTH, SCREEN_HEIGHT)
                );
            }
        }
    }

    impl Visible for Vec<Object> {
        fn show<Display: Scribe>(&self, display: &mut Display) {
            for object in self.iter().filter(|&o| !o.is_dynamic()) {
                object.show(display);
            }
        }
    }

    impl Visible for (&Texture<'_>, &Room) {
        fn show<Display: Scribe>(&self, display: &mut Display) {
            (self.0, self.1.tile_order).show(display);
            self.1.objects.show(display);
        }
    }

    impl Visible for sdl2::pixels::Color {
        fn show<Display: Scribe>(&self, display: &mut Display) {
            display.clear(*self)
        }
    }

    impl Visible for (&Surface<'_>, &Room) {
        fn show<Display: Scribe>(&self, display: &mut Display) {
            let loader = display.get_builder();
            let tx = loader.load(self.0).expect("Could not generate texture version of surface");
            (&tx, self.1).show(display)
        }
    }

    impl Visible for Texture<'_> {
        fn show<Display: Scribe>(&self, display: &mut Display) {
            display.draw(self, None, None)
        }
    }

    impl Visible for (&glider::Play, &Animations) {
        fn show<Display: Scribe>(&self, display: &mut Display) {
            let &(play, animations) = self;
            let (player_position, mode) = play.player();
            let (block, frame, clip) = match mode {
                Player::Flying { facing, backward } => (
                    match facing {Some(Side::Left) => "glider.left", Some(Side::Right) => "glider.right", _ => "glider.turn"}, 
                    animations.check(0).unwrap_or(if backward {atlas::TIPPED} else {atlas::LEVEL}),
                    None,
                ),
                Player::Shredding{ height } => ("glider.shreds", atlas::BITS, Some(height)),
            };
            if block == "glider.left" || block == "glider.right" {
                display.sprite((player_position.0, VERT_FLOOR as i16), TOP, block, atlas::SHADOW);
            }
            let items = play.visible_items().filter(
                |&(_, o)| {
                    if let object::Kind::Mirror(size) = o.kind {
                        let size = size - (8, 8);
                        let bounds = space::Rect::from(size / CENTER << *o.position);
                        display.clipping(bounds, |display|
                            display.sprite((player_position.0 - 16, player_position.1 - 32), CENTER, block, frame)
                        );
                        return false;
                    };
                    o.is_dynamic()
                }
            ).collect::<Vec<_>>();

            for (id, item) in items.into_iter().filter(|&(_, o)| o.is_dynamic()) {
                if let object::Kind::Toast(object::Motion{limit: Range{start, ..}, ..}, _) = item.kind {
                    if start > 0 {
                        animations.borrow_mut().remove(&id.get());
                    } else {
                        animations.borrow_mut().try_insert(id.get(), Box::new(atlas::TOAST.cycle().flat_map(|i| repeat(i).take(2)))).ok();
                    }
                }
                let frame = animations.check(id.get());
                (frame, item).show(display);
            }
            #[cfg(debug_assertions)]
            if crate::show_debug() {
                for frame in play.debug_zones() {
                    display.fill((0, 255, 0, 100), space::Rect::from(frame).into()).ok();
                }
            }
            match clip {
                Some(height) => {
                    if height > 0 {
                        display.clipping(
                            Rect::new(player_position.0 as i32 - 24, player_position.1 as i32, 48, height as u32),
                            |display| display.sprite((player_position.0, player_position.1 + height as i16), BOTTOM, block, frame)
                        )
                    }
                },
                None => display.sprite((player_position.0, player_position.1 + 10), BOTTOM, block, frame),
            } 
            display.publish();
        }
    }
}

trait Texturizer {
    fn load(&self, source: &Surface) -> Result<Texture, String>;
}

impl<T> Texturizer for TextureCreator<T> {
    fn load(&self, source: &Surface) -> Result<Texture, String> {
        source.as_texture(self).map_err(|e| e.to_string())
    }
}

trait Illuminator {
    type Builder: Texturizer;
    fn get_builder(&self) -> Self::Builder;
}

impl Illuminator for Canvas<Window> {
    type Builder = TextureCreator<sdl2::video::WindowContext>;
    fn get_builder(&self) -> Self::Builder { self.texture_creator() }
}

impl<'s> Illuminator for Canvas<Surface<'s>> {
    type Builder = TextureCreator<sdl2::surface::SurfaceContext<'s>>;
    fn get_builder(&self) -> Self::Builder { self.texture_creator() }
}

impl<I: Illuminator, T> Illuminator for (&mut I, T) {
    type Builder = <I as Illuminator>::Builder;
    fn get_builder(&self) -> Self::Builder { self.0.get_builder() }
}

#[allow(private_bounds)]
pub trait Scribe : Illuminator {
    fn clear(&mut self, color: sdl2::pixels::Color);
    fn publish(&mut self);
    fn clipping(&mut self, clip: impl Into<Option<Rect>>, doing: impl FnOnce(&mut Self));
    fn draw(&mut self, pixels: &Texture, source: impl Into<Option<Rect>>, dest: impl Into<Option<Rect>>);
    fn pen<const N: usize>(&mut self, stroke: impl Into<Color>, vertices: &[(i32, i32); N]) -> Result<(), String>;
    fn fill(&mut self, tone: impl Into<Color>, bounds: Rect) -> Result<(), String>;
    fn outline_rect(&mut self, bounds: Rect, fill: impl Into<Color>) -> Result<(), String>;
    fn limn_rect(&mut self, bounds: Rect, fill: impl Into<Color>, hilite: impl Into<Color>) -> Result<(), String>;
    fn sink_rect(&mut self, bounds: Rect, fill: impl Into<Option<Color>>) -> Result<(), String>;
    fn show<V: Visible>(&mut self, item: &V) -> &mut Self;
    fn sprite(&mut self, position: (i16, i16), anchor: Anchor, name: &str, index: usize);
}

impl<R:RenderTarget> Scribe for (&mut Canvas<R>, &Atlas<Texture<'_>>) where Self: Illuminator {
    fn clear(&mut self, color: sdl2::pixels::Color) {
        let display = &mut *self.0;
        display.set_draw_color(color);
        display.clear();
    }
    fn publish(&mut self) { self.0.present() }

    fn clipping(&mut self, clip: impl Into<Option<Rect>>, doing: impl FnOnce(&mut Self)) {
        self.0.set_clip_rect(clip.into());
        doing(self);
        self.0.set_clip_rect(None);
    }

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

    fn show<V: Visible>(&mut self, item: &V) -> &mut Self { item.show(self); self }

    fn sprite(&mut self, position: (i16, i16), anchor: Anchor, name: &str, index: usize) {
        let (wedge, tex) = self.1.get(name);
        let frame = wedge[index];
        let size = Bounds::from(frame).size();
        let bounds = space::Rect::from(size / anchor << *Reference::from(position));
        self.draw(tex, wedge[index], bounds);
    }
}
