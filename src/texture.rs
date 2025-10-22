use std::io::{BufReader, Cursor};

use base64::Engine;
use gl::types::GLenum;
use png::{ColorType, Decoder};

#[derive(Debug)]
pub enum DecodeError {
    Base64DecodeError(base64::DecodeError),
    PngDecodingError(png::DecodingError),
}

impl From<base64::DecodeError> for DecodeError {
    fn from(value: base64::DecodeError) -> Self {
        Self::Base64DecodeError(value)
    }
}

impl From<png::DecodingError> for DecodeError {
    fn from(value: png::DecodingError) -> Self {
        Self::PngDecodingError(value)
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base64DecodeError(err) => write!(f, "{}", err),
            Self::PngDecodingError(err) => write!(f, "{}", err),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Texture {
    format: ColorType,
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Texture {
    pub const GL_DATA_TYPE: GLenum = gl::UNSIGNED_BYTE;

    pub fn from_href(href: &str) -> Result<Self, DecodeError> {
        let decoded_image = Texture::decode_base64_encoded_image(href)?;

        let decoder = Decoder::new(BufReader::new(Cursor::new(decoded_image)));
        let mut reader = decoder.read_info()?;

        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf)?;

        Ok(Self {
            format: info.color_type,
            width: info.width,
            height: info.height,
            data: buf,
        })
    }

    pub fn gl_internal_format(&self) -> GLenum {
        match self.format {
            ColorType::Grayscale => gl::R8,
            ColorType::Rgb => gl::RGB,
            ColorType::Indexed => gl::R8,
            ColorType::GrayscaleAlpha => gl::RG8,
            ColorType::Rgba => gl::RGBA,
        }
    }

    pub fn gl_input_format(&self) -> GLenum {
        match self.format {
            ColorType::Grayscale => gl::RED,
            ColorType::Rgb => gl::RGB,
            ColorType::Indexed => gl::RED,
            ColorType::GrayscaleAlpha => gl::RG,
            ColorType::Rgba => gl::RGBA,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn decode_base64_encoded_image(contents: &str) -> Result<Vec<u8>, base64::DecodeError> {
        let contents = if let Some(index) = contents.chars().position(|c| c == ',') {
            &contents[index + ','.len_utf8()..]
        } else {
            contents
        };

        let contents = contents.replace(&[' ', '\t', '\n', '\r'][..], "");
        Ok(base64::prelude::BASE64_STANDARD.decode(contents)?)
    }
}
