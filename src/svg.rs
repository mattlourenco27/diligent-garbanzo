use std::{
    fs::File, io::BufReader, num::ParseFloatError, path::Path, str::FromStr, string::FromUtf8Error,
};

use quick_xml::{
    events::{attributes::Attributes, BytesEnd, BytesStart, Event},
    NsReader,
};

use crate::{
    color::{self, Color},
    texture::Texture,
    vector::{StaticVector, Vector2D},
};

#[derive(Debug)]
pub enum ReadError {
    EndTagBeforeStart,
    FromUtf8Error(FromUtf8Error),
    MissingSVGTag,
    ParseFloatError(ParseFloatError),
    XMLError(quick_xml::errors::Error),
}

#[derive(Debug)]
enum EventStatus {
    Eof,
    Error(ReadError),
    SkippedTag,
    UnrecognizedTag(String),
}

impl From<ReadError> for EventStatus {
    fn from(value: ReadError) -> Self {
        Self::Error(value)
    }
}

impl From<quick_xml::errors::Error> for EventStatus {
    fn from(value: quick_xml::errors::Error) -> Self {
        Self::Error(ReadError::XMLError(value))
    }
}

impl From<quick_xml::errors::Error> for ReadError {
    fn from(value: quick_xml::errors::Error) -> Self {
        Self::XMLError(value)
    }
}

impl From<quick_xml::events::attributes::AttrError> for ReadError {
    fn from(value: quick_xml::events::attributes::AttrError) -> Self {
        Self::XMLError(value.into())
    }
}

impl From<FromUtf8Error> for EventStatus {
    fn from(value: FromUtf8Error) -> Self {
        Self::Error(ReadError::FromUtf8Error(value))
    }
}

impl From<ParseFloatError> for ReadError {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndTagBeforeStart => write!(
                f,
                "An end tag was found before it's corresponding start tag"
            ),
            Self::FromUtf8Error(err) => write!(f, "Could not convert to UTF-8: {}", err),
            Self::MissingSVGTag => write!(f, "Could not find an svg tag at the top level"),
            Self::ParseFloatError(err) => write!(f, "Could not parse float: {}", err),
            Self::XMLError(err) => write!(f, "XML Error: {}", err),
        }
    }
}

impl std::fmt::Display for EventStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eof => write!(f, "Reached end of file"),
            Self::Error(err) => err.fmt(f),
            Self::SkippedTag => write!(f, "Tag was not read (skipped)"),
            Self::UnrecognizedTag(err) => write!(f, "Unrecognized tag: {}", err),
        }
    }
}

#[derive(Debug)]
enum Element {
    EmptyTag(EmptyTag),
    EndTag(EndTag),
    StartTag(StartTag),
}

#[derive(Debug)]
enum EmptyTag {
    Ellipse(Ellipse),
    Image(Image),
    Line(Line),
    Point(Point),
    Polygon(Polygon),
    Polyline(Polyline),
    Rect(Rect),
}

impl EmptyTag {
    fn from_empty_tag_bytes(bytes: BytesStart) -> Result<EmptyTag, EventStatus> {
        match bytes.local_name().into_inner() {
            b"point" => Ok(EmptyTag::Point(Point::from_bytes_start(bytes)?)),
            b"line" => Ok(EmptyTag::Line(Line::from_bytes_start(bytes)?)),
            b"polyline" => Ok(EmptyTag::Polyline(Polyline::from_bytes_start(bytes)?)),
            b"rect" => Ok(Rect::from_bytes_start(bytes)?),
            b"polygon" => Ok(EmptyTag::Polygon(Polygon::from_bytes_start(bytes)?)),
            b"ellipse" => Ok(EmptyTag::Ellipse(Ellipse::from_bytes_start(bytes)?)),
            b"image" => unimplemented!(),
            unrecognized => Err(EventStatus::UnrecognizedTag(String::from_utf8(
                unrecognized.to_owned(),
            )?)),
        }
    }
}

#[derive(Debug, PartialEq)]
enum EndTag {
    Group,
    SVG,
}

impl EndTag {
    fn from_end_tag_bytes(bytes: BytesEnd) -> Result<EndTag, EventStatus> {
        match bytes.local_name().into_inner() {
            b"g" => Ok(EndTag::Group),
            b"svg" => Ok(EndTag::SVG),
            unrecognized => Err(EventStatus::UnrecognizedTag(String::from_utf8(
                unrecognized.to_owned(),
            )?)),
        }
    }
}

#[derive(Debug)]
enum StartTag {
    Group(Group),
    SVG(SVG),
}

impl StartTag {
    fn get_expected_end_tag(&self) -> EndTag {
        match self {
            StartTag::Group(..) => EndTag::Group,
            StartTag::SVG(..) => EndTag::SVG,
        }
    }

    fn add_element(&mut self, element: Element) {
        match self {
            StartTag::Group(group) => group.elements.push(element),
            StartTag::SVG(svg) => svg.elements.push(element),
        }
    }

    fn from_start_tag_bytes(bytes: BytesStart) -> Result<Self, EventStatus> {
        match bytes.local_name().into_inner() {
            b"g" => unimplemented!(),
            b"svg" => Ok(StartTag::SVG(SVG::from_bytes_start(bytes)?)),
            unrecognized => Err(EventStatus::UnrecognizedTag(String::from_utf8(
                unrecognized.to_owned(),
            )?)),
        }
    }
}

#[derive(Debug)]
struct Point {
    style: Style,
    position: Vector2D<f64>,
}

impl Point {
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, ReadError> {
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
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, ReadError> {
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
    points: Vec<Vector2D<f64>>,
}

impl Polyline {
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, ReadError> {
        let style = Style::from_attributes(bytes.attributes().clone())?;

        let mut points = Vec::new();

        for attribute in bytes.attributes() {
            let attribute = attribute?;
            match attribute.key.local_name().into_inner() {
                b"points" => {
                    for point_str in attribute.unescape_value()?.split_whitespace() {
                        let (x_str, y_str) = point_str.split_once(',').unwrap();
                        let x = f64::from_str(x_str)?;
                        let y = f64::from_str(y_str)?;
                        points.push(StaticVector([x, y]))
                    }
                }
                _ => (),
            };
        }

        Ok(Self { style, points })
    }
}

#[derive(Debug)]
struct Rect {
    style: Style,
    position: Vector2D<f64>,
    dimension: Vector2D<f64>,
}

impl Rect {
    fn from_bytes_start(bytes: BytesStart) -> Result<EmptyTag, ReadError> {
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
            return Ok(EmptyTag::Point(Point {
                style,
                position: StaticVector([x, y]),
            }));
        }

        Ok(EmptyTag::Rect(Rect {
            style,
            position: StaticVector([x, y]),
            dimension: StaticVector([width, height]),
        }))
    }
}

#[derive(Debug)]
struct Polygon {
    style: Style,
    points: Vec<Vector2D<f64>>,
}

impl Polygon {
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, ReadError> {
        let style = Style::from_attributes(bytes.attributes().clone())?;

        let mut points = Vec::new();

        for attribute in bytes.attributes() {
            let attribute = attribute?;
            match attribute.key.local_name().into_inner() {
                b"points" => {
                    for point_str in attribute.unescape_value()?.split_whitespace() {
                        let (x_str, y_str) = point_str.split_once(',').unwrap();
                        let x = f64::from_str(x_str)?;
                        let y = f64::from_str(y_str)?;
                        points.push(StaticVector([x, y]))
                    }
                }
                _ => (),
            };
        }

        Ok(Self { style, points })
    }
}

#[derive(Debug)]
struct Ellipse {
    style: Style,
    center: Vector2D<f64>,
    radius: Vector2D<f64>,
}

impl Ellipse {
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, ReadError> {
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
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, ReadError> {
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
    fn from_attributes(attributes: Attributes) -> Result<Self, ReadError> {
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

fn read_next_event(reader: &mut NsReader<BufReader<File>>) -> Result<Element, EventStatus> {
    let mut buf = Vec::new();

    let next_event = reader.read_event_into(&mut buf)?;
    match next_event {
        Event::Start(start_tag_bytes) => Ok(Element::StartTag(StartTag::from_start_tag_bytes(
            start_tag_bytes,
        )?)),
        // Event::Text(event) => unimplemented!(),
        Event::End(end_tag_bytes) => {
            Ok(Element::EndTag(EndTag::from_end_tag_bytes(end_tag_bytes)?))
        }
        Event::Empty(empty_tag_bytes) => Ok(Element::EmptyTag(EmptyTag::from_empty_tag_bytes(
            empty_tag_bytes,
        )?)),
        Event::Eof => Err(EventStatus::Eof),
        _ => Err(EventStatus::SkippedTag),
    }
}

fn handle_next_element(
    tag_lifo: &mut Vec<StartTag>,
    element: Element,
) -> Result<Option<SVG>, ReadError> {
    match element {
        Element::EmptyTag(..) => match tag_lifo.last_mut() {
            None => Err(ReadError::MissingSVGTag),
            Some(last) => {
                last.add_element(element);
                Ok(None)
            }
        },
        Element::StartTag(start_tag) => {
            tag_lifo.push(start_tag);
            Ok(None)
        }
        Element::EndTag(end_tag) => {
            let completed_element = match tag_lifo.pop() {
                None => return Err(ReadError::EndTagBeforeStart),
                Some(last) => {
                    if end_tag != last.get_expected_end_tag() {
                        return Err(ReadError::EndTagBeforeStart);
                    }
                    last
                }
            };

            match tag_lifo.last_mut() {
                None => match completed_element {
                    StartTag::Group(..) => Err(ReadError::MissingSVGTag),
                    StartTag::SVG(svg) => Ok(Some(svg)),
                },
                Some(last) => {
                    last.add_element(Element::StartTag(completed_element));
                    Ok(None)
                }
            }
        }
    }
}

pub fn read_from_file(path: &Path) -> Result<SVG, ReadError> {
    let mut reader = NsReader::from_file(path)?;

    let mut tag_lifo = Vec::new();

    loop {
        match read_next_event(&mut reader) {
            Ok(element) => match handle_next_element(&mut tag_lifo, element)? {
                Some(svg) => return Ok(svg),
                None => (),
            },
            Err(status) => match status {
                EventStatus::Error(err) => return Err(err),
                EventStatus::UnrecognizedTag(_) => println!("{}", status),
                EventStatus::SkippedTag => (),
                EventStatus::Eof => break,
            },
        };
    }

    Err(ReadError::MissingSVGTag)
}
