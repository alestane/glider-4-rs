
pub mod color {
    use std::collections::HashMap;
    const SPRITES: &[u8] = include_bytes!("color/128.png");
    const ROOM_200: &[u8] = include_bytes!("color/200.png");
    const ROOM_201: &[u8] = include_bytes!("color/201.png");
    const ROOM_202: &[u8] = include_bytes!("color/202.png");

    pub fn assets() -> HashMap<usize, &'static [u8]> {
        let mut values = HashMap::<usize, &'static [u8]>::new();
        values.insert(128, SPRITES);
        values.insert(200, ROOM_200);
        values.insert(201, ROOM_201);
        values.insert(202, ROOM_202);
        values
    }
}

pub const CIRCLE: &'static [u8] = include_bytes!("circle.raw");

pub const THE_HOUSE: &'static [u8] = include_bytes!("The House");