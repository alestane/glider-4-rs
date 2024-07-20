
pub mod color {
    use std::collections::HashMap;
    const SPRITES: &[u8] = include_bytes!("color/128.png");
    const ROOM_200: &[u8] = include_bytes!("color/200.png");
    const ROOM_201: &[u8] = include_bytes!("color/201.png");
    const ROOM_202: &[u8] = include_bytes!("color/202.png");
    const ROOM_204: &[u8] = include_bytes!("color/204.png");
    const ROOM_206: &[u8] = include_bytes!("color/206.png");
    const ROOM_207: &[u8] = include_bytes!("color/207.png");

    pub fn assets() -> HashMap<usize, &'static [u8]> {
        HashMap::from_iter(
            [
                (128, SPRITES),
                (200, ROOM_200),
                (201, ROOM_201),
                (202, ROOM_202),
                (204, ROOM_204),
                (206, ROOM_206),
                (207, ROOM_207),
            ]
        )
    }
}

pub const CIRCLE: &'static [u8] = include_bytes!("circle.raw");

pub const THE_HOUSE: &'static [u8] = include_bytes!("The House");