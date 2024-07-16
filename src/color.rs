use hex::FromHex;

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub const NONE: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        if value == "none" || value.len() == 0 {
            return NONE;
        }

        let hex = value.strip_prefix('#').unwrap_or(value);

        const INV: f32 = 1.0 / core::u8::MAX as f32;
        if hex.len() == 6 {
            let bytes = match <[u8; 3]>::from_hex(hex) {
                Ok(bytes) => bytes,
                Err(_) => return NONE,
            };
            return Color {
                r: bytes[0] as f32 * INV,
                g: bytes[1] as f32 * INV,
                b: bytes[2] as f32 * INV,
                a: 1.0,
            };
        }

        if hex.len() == 8 {
            let bytes = match <[u8; 4]>::from_hex(hex) {
                Ok(bytes) => bytes,
                Err(_) => return NONE,
            };
            return Color {
                r: bytes[0] as f32 * INV,
                g: bytes[1] as f32 * INV,
                b: bytes[2] as f32 * INV,
                a: bytes[3] as f32 * INV,
            };
        }

        NONE
    }
}
