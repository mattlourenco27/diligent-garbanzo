use hex::FromHex;

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

const NONE: Color = Color {
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

impl From<&[u8]> for Color {
    fn from(value: &[u8]) -> Self {
        if value == b"none" || value.len() == 0 {
            return NONE;
        }
        
        let hex_bytes;

        let has_leading_pound = value[0] == b'#';
        if has_leading_pound {
            hex_bytes = &value[1..];
        } else {
            hex_bytes = value;
        }
        
        const INV: f32 = 1.0 / std::u8::MAX as f32;
        if hex_bytes.len() == 6 {
            let buffer = match <[u8; 3]>::from_hex(hex_bytes) {
                Ok(buffer) => buffer,
                Err(_) => return NONE,
            };
            return Color {
                r : buffer[0] as f32 * INV,
                g : buffer[1] as f32 * INV,
                b : buffer[2] as f32 * INV,
                a : 1.0,
            }
        }

        if hex_bytes.len() == 8 {
            let buffer = match <[u8; 4]>::from_hex(hex_bytes) {
                Ok(buffer) => buffer,
                Err(_) => return NONE,
            };
            return Color {
                r : buffer[0] as f32 * INV,
                g : buffer[1] as f32 * INV,
                b : buffer[2] as f32 * INV,
                a : buffer[3] as f32 * INV,
            }
        }

        return NONE;
    }
}
