use std::{
    fs::File, io::BufReader, num::ParseFloatError, path::Path, str::FromStr, string::FromUtf8Error,
};

use quick_xml::{
    events::{attributes::Attributes, BytesStart, Event},
    NsReader,
};

use crate::{
    color::{self, Color},
    texture::Texture,
    vector::{StaticVector, Vector2D},
};

#[derive(Debug)]
pub enum Error {
    XMLError(quick_xml::errors::Error),
    FromUtf8Error(FromUtf8Error),
    ParseFloatError(ParseFloatError),
    UnrecognizedTag(String),
}

impl From<quick_xml::errors::Error> for Error {
    fn from(value: quick_xml::errors::Error) -> Self {
        return Self::XMLError(value);
    }
}

impl From<quick_xml::events::attributes::AttrError> for Error {
    fn from(value: quick_xml::events::attributes::AttrError) -> Self {
        return Self::XMLError(value.into());
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        return Self::FromUtf8Error(value);
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        return Self::ParseFloatError(value);
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::XMLError(err) => write!(f, "XML Error: {}", err),
            Self::FromUtf8Error(err) => write!(f, "Could not convert to UTF-8: {}", err),
            Self::ParseFloatError(err) => write!(f, "Could not parse float: {}", err),
            Self::UnrecognizedTag(err) => write!(f, "Unrecognized tag: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::XMLError(err) => Some(err),
            Self::FromUtf8Error(err) => Some(err),
            Self::ParseFloatError(err) => Some(err),
            Self::UnrecognizedTag(_) => None,
        }
    }
}

#[derive(Debug)]
enum Element {
    None,
    Point(Point),
    Line(Line),
    Polyline(Polyline),
    Rect(Rect),
    Polygon(Polygon),
    Ellipse(Ellipse),
    Image(Image),
    Group(Group),
    SVG(SVG),
}

#[derive(Debug)]
struct Point {
    style: Style,
    position: Vector2D<f64>,
}

impl Point {
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, Error> {
        let style = Style::from_attributes(bytes.attributes().clone())?;

        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;

        for attribute in bytes.attributes() {
            let attribute = attribute?;
            match attribute.key.local_name().into_inner() {
                b"x" => x = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"y" => y = f64::from_str(attribute.unescape_value()?.as_ref())?,
                _ => (),
            };
        }

        Ok(Self {
            style,
            position: StaticVector([x, y]),
        })
    }
}

#[derive(Debug)]
struct Line {
    style: Style,
    from: Vector2D<f64>,
    to: Vector2D<f64>,
}

impl Line {
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, Error> {
        let style = Style::from_attributes(bytes.attributes().clone())?;

        let mut x1: f64 = 0.0;
        let mut y1: f64 = 0.0;
        let mut x2: f64 = 0.0;
        let mut y2: f64 = 0.0;

        for attribute in bytes.attributes() {
            let attribute = attribute?;
            match attribute.key.local_name().into_inner() {
                b"x1" => x1 = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"y1" => y1 = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"x2" => x2 = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"y2" => y2 = f64::from_str(attribute.unescape_value()?.as_ref())?,
                _ => (),
            };
        }

        Ok(Self {
            style,
            from: StaticVector([x1, y1]),
            to: StaticVector([x2, y2]),
        })
    }
}

#[derive(Debug)]
struct Polyline {
    style: Style,
    position: Vec<Vector2D<f64>>,
}

#[derive(Debug)]
struct Rect {
    style: Style,
    position: Vector2D<f64>,
    dimension: Vector2D<f64>,
}

impl Rect {
    fn from_bytes_start(bytes: BytesStart) -> Result<Element, Error> {
        let style = Style::from_attributes(bytes.attributes().clone())?;

        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        let mut width: f64 = 0.0;
        let mut height: f64 = 0.0;

        for attribute in bytes.attributes() {
            let attribute = attribute?;
            match attribute.key.local_name().into_inner() {
                b"x" => x = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"y" => y = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"width" => width = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"height" => height = f64::from_str(attribute.unescape_value()?.as_ref())?,
                _ => (),
            };
        }

        if width == 0.0 && height == 0.0 {
            return Ok(Element::Point(Point {
                style,
                position: StaticVector([x, y]),
            }));
        }

        Ok(Element::Rect(Rect {
            style,
            position: StaticVector([x, y]),
            dimension: StaticVector([width, height]),
        }))
    }
}

#[derive(Debug)]
struct Polygon {
    style: Style,
    position: Vec<Vector2D<f64>>,
}

#[derive(Debug)]
struct Ellipse {
    style: Style,
    center: Vector2D<f64>,
    radius: Vector2D<f64>,
}

impl Ellipse {
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, Error> {
        let style = Style::from_attributes(bytes.attributes().clone())?;

        let mut cx: f64 = 0.0;
        let mut cy: f64 = 0.0;
        let mut rx: f64 = 0.0;
        let mut ry: f64 = 0.0;

        for attribute in bytes.attributes() {
            let attribute = attribute?;
            match attribute.key.local_name().into_inner() {
                b"cx" => cx = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"cy" => cy = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"rx" => rx = f64::from_str(attribute.unescape_value()?.as_ref())?,
                b"ry" => ry = f64::from_str(attribute.unescape_value()?.as_ref())?,
                _ => (),
            };
        }

        Ok(Self {
            style,
            center: StaticVector([cx, cy]),
            radius: StaticVector([rx, ry]),
        })
    }
}

#[derive(Debug)]
struct Image {
    style: Style,
    position: Vector2D<f64>,
    dimension: Vector2D<f64>,
    texture: Texture,
}

#[derive(Debug)]
struct Group {
    style: Style,
    elements: Vec<Element>,
}

#[derive(Debug)]
pub struct SVG {
    dimension: Vector2D<f64>,
    elements: Vec<Element>,
}

impl SVG {
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, Error> {
        let mut width: f64 = 300.0;
        let mut height: f64 = 150.0;

        for attribute in bytes.attributes() {
            let attribute = attribute?;
            match attribute.key.local_name().into_inner() {
                b"height" => height = pixels_to_dim(attribute.unescape_value()?.as_ref())?,
                b"width" => width = pixels_to_dim(attribute.unescape_value()?.as_ref())?,
                _ => (),
            };
        }

        Ok(Self {
            dimension: StaticVector([width, height]),
            elements: Vec::new(),
        })
    }
}

#[derive(Debug)]
struct Style {
    stroke_color: Color,
    fill_color: Color,
    stroke_width: f64,
    miter_limit: f64,
}

impl Style {
    fn from_attributes(attributes: Attributes) -> Result<Self, Error> {
        let mut stroke_color: Color = color::NONE;
        let mut fill_color: Color = color::NONE;
        let mut stroke_width: f64 = 1.0;
        let mut miter_limit: f64 = 4.0;

        for attribute in attributes {
            let attribute = attribute?;
            match attribute.key.local_name().into_inner() {
                b"fill" => fill_color = Color::from(attribute.unescape_value()?.as_ref()),
                b"fill-opacity" => {
                    fill_color.a = f32::from_str(attribute.unescape_value()?.as_ref())?
                }
                b"stroke" => stroke_color = Color::from(attribute.unescape_value()?.as_ref()),
                b"stroke-opacity" => {
                    stroke_color.a = f32::from_str(attribute.unescape_value()?.as_ref())?
                }
                b"stroke-width" => {
                    stroke_width = f64::from_str(attribute.unescape_value()?.as_ref())?
                }
                b"stroke-miterlimit" => {
                    miter_limit = f64::from_str(attribute.unescape_value()?.as_ref())?
                }
                b"transform" => unimplemented!(),
                _ => (),
            };
        }

        Ok(Self {
            stroke_color,
            fill_color,
            stroke_width,
            miter_limit,
        })
    }
}

fn pixels_to_dim(pixels: &str) -> Result<f64, ParseFloatError> {
    let mut dim_str = pixels.trim();
    dim_str = dim_str.strip_suffix("px").unwrap_or(dim_str);
    f64::from_str(dim_str)
}

fn handle_start_tag_bytes(bytes: BytesStart) -> Result<Element, Error> {
    match bytes.local_name().into_inner() {
        b"svg" => Ok(Element::SVG(SVG::from_bytes_start(bytes)?)),
        unrecognized => Err(Error::UnrecognizedTag(String::from_utf8(
            unrecognized.to_owned(),
        )?)),
    }
}

fn handle_empty_tag_bytes(bytes: BytesStart) -> Result<Element, Error> {
    match bytes.local_name().into_inner() {
        b"point" => Ok(Element::Point(Point::from_bytes_start(bytes)?)),
        b"line" => Ok(Element::Line(Line::from_bytes_start(bytes)?)),
        b"polyline" => unimplemented!(),
        b"rect" => Ok(Rect::from_bytes_start(bytes)?),
        b"polygon" => unimplemented!(),
        b"ellipse" => Ok(Element::Ellipse(Ellipse::from_bytes_start(bytes)?)),
        b"image" => unimplemented!(),
        b"group" => unimplemented!(),
        unrecognized => Err(Error::UnrecognizedTag(String::from_utf8(
            unrecognized.to_owned(),
        )?)),
    }
}

fn read_next_event(reader: &mut NsReader<BufReader<File>>) -> Result<Element, Error> {
    let mut buf = Vec::new();

    let next_event = reader.read_event_into(&mut buf)?;
    match next_event {
        Event::Start(event) => handle_start_tag_bytes(event),
        Event::Text(event) => unimplemented!(),
        Event::End(event) => unimplemented!(),
        Event::Empty(event) => handle_empty_tag_bytes(event),
        Event::Eof => Ok(Element::None),
        _ => Ok(Element::None),
    }
}

pub fn read_from_file(path: &Path) -> Result<SVG, Error> {
    let mut reader = NsReader::from_file(path)?;

    let ret = SVG {
        dimension: StaticVector([0.0, 0.0]),
        elements: Vec::new(),
    };

    loop {
        let element = read_next_event(&mut reader)?;
        match element {
            Element::None => return Ok(ret),
            other => dbg!(other),
        };
    }
}
