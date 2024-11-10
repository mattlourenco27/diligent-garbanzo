use std::{
    borrow::Cow, fs::File, io::BufReader, num::ParseFloatError, path::Path, str::FromStr,
    string::FromUtf8Error,
};

use hex::FromHex;
use once_cell::sync;
use quick_xml::{
    events::{BytesEnd, BytesStart, Event},
    NsReader,
};

use regex::Regex;
use sdl2::pixels::Color;

use crate::{matrix::Matrix3x3, texture::Texture, vector::Vector2D};

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
pub enum Element {
    EmptyTag(EmptyTag),
    EndTag(EndTag),
    StartTag(StartTag),
}

#[derive(Debug)]
pub enum EmptyTag {
    Ellipse(Ellipse),
    Image(Image),
    Line(Line),
    Point(Point),
    Polygon(Polygon),
    Polyline(Polyline),
    Rect(Rect),
}

impl EmptyTag {
    fn from_empty_tag_bytes(
        bytes: BytesStart,
        parent_style: Style,
    ) -> Result<EmptyTag, EventStatus> {
        match bytes.local_name().into_inner() {
            b"point" => Ok(EmptyTag::Point(Point::from_bytes_start(
                bytes,
                parent_style,
            )?)),
            b"line" => Ok(EmptyTag::Line(Line::from_bytes_start(bytes, parent_style)?)),
            b"polyline" => Ok(EmptyTag::Polyline(Polyline::from_bytes_start(
                bytes,
                parent_style,
            )?)),
            b"rect" => Ok(Rect::from_bytes_start(bytes, parent_style)?),
            b"polygon" => Ok(EmptyTag::Polygon(Polygon::from_bytes_start(
                bytes,
                parent_style,
            )?)),
            b"ellipse" => Ok(EmptyTag::Ellipse(Ellipse::from_bytes_start(
                bytes,
                parent_style,
            )?)),
            b"image" => unimplemented!(),
            unrecognized => Err(EventStatus::UnrecognizedTag(String::from_utf8(
                unrecognized.to_owned(),
            )?)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EndTag {
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
pub enum StartTag {
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

    fn from_start_tag_bytes(
        bytes: BytesStart,
        parent_style: Style,
    ) -> Result<(Self, Style), EventStatus> {
        match bytes.local_name().into_inner() {
            b"g" => {
                let group = Group::from_bytes_start(bytes, parent_style)?;
                let style = group.style.clone();
                Ok((StartTag::Group(group), style))
            }
            b"svg" => Ok((StartTag::SVG(SVG::from_bytes_start(bytes)?), Style::DEFAULT)),
            unrecognized => Err(EventStatus::UnrecognizedTag(String::from_utf8(
                unrecognized.to_owned(),
            )?)),
        }
    }
}

#[derive(Debug)]
struct Attribute<'a> {
    pub key: &'a [u8],
    value: Cow<'a, str>,
}

impl<'a> Attribute<'a> {
    fn parse(attribute: quick_xml::events::attributes::Attribute<'a>) -> Result<Self, ReadError> {
        Ok(Attribute {
            key: attribute.key.local_name().into_inner(),
            value: attribute.unescape_value()?,
        })
    }

    fn color(&self) -> Color {
        let value = self.value.as_ref();

        if value == "none" || value.len() == 0 {
            return Style::COLOR_NONE;
        }

        let hex = value.strip_prefix('#').unwrap_or(value);

        if hex.len() == 6 {
            let bytes = match <[u8; 3]>::from_hex(hex) {
                Ok(bytes) => bytes,
                Err(_) => return Style::COLOR_NONE,
            };
            return Color {
                r: bytes[0],
                g: bytes[1],
                b: bytes[2],
                a: core::u8::MAX,
            };
        }

        if hex.len() == 8 {
            let bytes = match <[u8; 4]>::from_hex(hex) {
                Ok(bytes) => bytes,
                Err(_) => return Style::COLOR_NONE,
            };
            return Color {
                r: bytes[0],
                g: bytes[1],
                b: bytes[2],
                a: bytes[3],
            };
        }

        Style::COLOR_NONE
    }

    fn length(&self) -> Result<f64, ReadError> {
        const SUPPORTED_UNITS: [(&str, f64); 7] = [
            ("cm", 9600.0 / 254.0),
            ("mm", 960.0 / 254.0),
            ("Q", 240.0 / 254.0),
            ("in", 96.0),
            ("pc", 16.0),
            ("pt", 96.0 / 72.0),
            ("px", 1.0),
        ];

        let trimmed_str = self.value.trim();
        let mut numeric_str = trimmed_str;
        let mut modifier = 1.0;
        for (unit, val_to_px) in SUPPORTED_UNITS.iter() {
            match trimmed_str.strip_suffix(unit) {
                None => continue,
                Some(value) => {
                    numeric_str = value;
                    modifier = *val_to_px;
                    break;
                }
            }
        }
        Ok(f64::from_str(numeric_str)? * modifier)
    }

    fn parse_number(raw_str: &str) -> Result<f64, ParseFloatError> {
        Ok(f64::from_str(raw_str.trim())?)
    }

    fn number(&self) -> Result<f64, ParseFloatError> {
        Attribute::parse_number(self.value.as_ref())
    }

    fn parse_number_list(raw_str: &str) -> Result<Vec<f64>, ParseFloatError> {
        static RE: sync::Lazy<Regex> =
            sync::Lazy::new(|| Regex::new(r"[,\s]+").expect("Invalid Regex"));

        let mut numbers = Vec::new();

        for float_str in RE.split(raw_str) {
            if !float_str.is_empty() {
                numbers.push(Attribute::parse_number(float_str)?)
            }
        }
        Ok(numbers)
    }

    fn number_list(&self) -> Result<Vec<f64>, ParseFloatError> {
        Attribute::parse_number_list(self.value.as_ref())
    }

    fn point_list(&self) -> Result<Vec<Vector2D<f64>>, ReadError> {
        let mut points = Vec::new();

        let mut x = 0.0;
        let mut y;
        for (i, value) in self.number_list()?.into_iter().enumerate() {
            if i % 2 == 0 {
                x = value;
            } else {
                y = value;
                points.push([x, y].into())
            }
        }

        Ok(points)
    }

    // This implements the SVG transformation specification. All the SVG
    // transformations are supported as documented in the link below:
    // https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute/transform
    fn transform_list(&self) -> Result<Matrix3x3<f64>, ParseFloatError> {
        const DEG_TO_RAD: f64 = core::f64::consts::PI / 180.0;

        static RE: sync::Lazy<Regex> =
            sync::Lazy::new(|| Regex::new(r"\)[,\s]*").expect("Invalid Regex"));

        let mut final_transform = Matrix3x3::identity();

        for transform_str in RE.split(self.value.as_ref()) {
            let (transform_type, values) = match transform_str.split_once('(') {
                None => continue,
                Some(ret) => ret,
            };

            let numbers = Attribute::parse_number_list(values)?;

            let transform: Matrix3x3<f64> = match transform_type {
                "matrix" => {
                    if numbers.len() != 6 {
                        continue;
                    }
                    [
                        [numbers[0], numbers[2], numbers[4]],
                        [numbers[1], numbers[3], numbers[5]],
                        [0.0, 0.0, 1.0],
                    ]
                    .into()
                }
                "translate" => {
                    if numbers.is_empty() || numbers.len() > 2 {
                        continue;
                    }
                    let x = numbers[0];
                    let y = *numbers.get(1).unwrap_or(&0.0);
                    [[1.0, 0.0, x], [0.0, 1.0, y], [0.0, 0.0, 1.0]].into()
                }
                "scale" => {
                    if numbers.is_empty() || numbers.len() > 2 {
                        continue;
                    }
                    let x = numbers[0];
                    let y = *numbers.get(1).unwrap_or(&x);
                    [[x, 0.0, 0.0], [0.0, y, 0.0], [0.0, 0.0, 1.0]].into()
                }
                "rotate" => {
                    if numbers.len() != 1 && numbers.len() != 3 {
                        continue;
                    }
                    let a = numbers[0] * DEG_TO_RAD;
                    let x = *numbers.get(1).unwrap_or(&0.0);
                    let y = *numbers.get(2).unwrap_or(&0.0);
                    let cx = -x * f64::cos(a) + y * f64::sin(a) + x;
                    let cy = -x * f64::sin(a) - y * f64::cos(a) + y;
                    [
                        [f64::cos(a), -f64::sin(a), cx],
                        [f64::sin(a), f64::cos(a), cy],
                        [0.0, 0.0, 1.0],
                    ]
                    .into()
                }
                "skewX" => {
                    if numbers.len() != 1 {
                        continue;
                    }
                    let a = numbers[0] * DEG_TO_RAD;
                    [[1.0, f64::tan(a), 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]].into()
                }
                "skewY" => {
                    if numbers.len() != 1 {
                        continue;
                    }
                    let a = numbers[0] * DEG_TO_RAD;
                    [[1.0, 0.0, 0.0], [f64::tan(a), 1.0, 0.0], [0.0, 0.0, 1.0]].into()
                }
                _ => continue,
            };

            final_transform *= transform;
        }

        Ok(final_transform)
    }
}

#[derive(Debug)]
pub struct Point {
    pub style: Style,
    pub position: Vector2D<f64>,
}

impl Point {
    fn from_bytes_start(bytes: BytesStart, parent_style: Style) -> Result<Self, ReadError> {
        let style = Style::from_attributes(bytes.attributes().clone(), parent_style)?;

        let mut x = 0.0;
        let mut y = 0.0;

        for attribute in bytes.attributes() {
            let attribute = Attribute::parse(attribute?)?;
            match attribute.key {
                b"x" => x = attribute.number()?,
                b"y" => y = attribute.number()?,
                _ => (),
            };
        }

        Ok(Self {
            style,
            position: [x, y].into(),
        })
    }
}

#[derive(Debug)]
pub struct Line {
    pub style: Style,
    pub from: Vector2D<f64>,
    pub to: Vector2D<f64>,
}

impl Line {
    fn from_bytes_start(bytes: BytesStart, parent_style: Style) -> Result<Self, ReadError> {
        let style = Style::from_attributes(bytes.attributes().clone(), parent_style)?;

        let mut x1 = 0.0;
        let mut y1 = 0.0;
        let mut x2 = 0.0;
        let mut y2 = 0.0;

        for attribute in bytes.attributes() {
            let attribute = Attribute::parse(attribute?)?;
            match attribute.key {
                b"x1" => x1 = attribute.length()?,
                b"y1" => y1 = attribute.length()?,
                b"x2" => x2 = attribute.length()?,
                b"y2" => y2 = attribute.length()?,
                _ => (),
            };
        }

        Ok(Self {
            style,
            from: [x1, y1].into(),
            to: [x2, y2].into(),
        })
    }
}

#[derive(Debug)]
pub struct Polyline {
    pub style: Style,
    pub points: Vec<Vector2D<f64>>,
}

impl Polyline {
    fn from_bytes_start(bytes: BytesStart, parent_style: Style) -> Result<Self, ReadError> {
        let style = Style::from_attributes(bytes.attributes().clone(), parent_style)?;

        let mut points = Vec::new();

        for attribute in bytes.attributes() {
            let attribute = Attribute::parse(attribute?)?;
            match attribute.key {
                b"points" => points = attribute.point_list()?,
                _ => (),
            };
        }

        Ok(Self { style, points })
    }
}

#[derive(Debug)]
pub struct Rect {
    pub style: Style,
    pub position: Vector2D<f64>,
    pub dimension: Vector2D<f64>,
    pub corners: Vector2D<f64>,
}

impl Rect {
    fn from_bytes_start(bytes: BytesStart, parent_style: Style) -> Result<EmptyTag, ReadError> {
        let style = Style::from_attributes(bytes.attributes().clone(), parent_style)?;

        let mut x = 0.0;
        let mut y = 0.0;
        let mut rx = 0.0;
        let mut ry = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;

        for attribute in bytes.attributes() {
            let attribute = Attribute::parse(attribute?)?;
            match attribute.key {
                b"x" => x = attribute.length()?,
                b"y" => y = attribute.length()?,
                b"rx" => rx = attribute.length()?,
                b"ry" => ry = attribute.length()?,
                b"width" => width = attribute.length()?,
                b"height" => height = attribute.length()?,
                _ => (),
            };
        }

        if width == 0.0 && height == 0.0 {
            return Ok(EmptyTag::Point(Point {
                style,
                position: [x, y].into(),
            }));
        }

        Ok(EmptyTag::Rect(Rect {
            style,
            position: [x, y].into(),
            dimension: [width, height].into(),
            corners: [rx, ry].into(),
        }))
    }
}

#[derive(Debug)]
pub struct Polygon {
    pub style: Style,
    pub points: Vec<Vector2D<f64>>,
}

impl Polygon {
    fn from_bytes_start(bytes: BytesStart, parent_style: Style) -> Result<Self, ReadError> {
        let style = Style::from_attributes(bytes.attributes().clone(), parent_style)?;

        let mut points = Vec::new();

        for attribute in bytes.attributes() {
            let attribute = Attribute::parse(attribute?)?;
            match attribute.key {
                b"points" => points = attribute.point_list()?,
                _ => (),
            };
        }

        Ok(Self { style, points })
    }
}

#[derive(Debug)]
pub struct Ellipse {
    pub style: Style,
    pub center: Vector2D<f64>,
    pub radius: Vector2D<f64>,
}

impl Ellipse {
    fn from_bytes_start(bytes: BytesStart, parent_style: Style) -> Result<Self, ReadError> {
        let style = Style::from_attributes(bytes.attributes().clone(), parent_style)?;

        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut rx = 0.0;
        let mut ry = 0.0;

        for attribute in bytes.attributes() {
            let attribute = Attribute::parse(attribute?)?;
            match attribute.key {
                b"cx" => cx = attribute.length()?,
                b"cy" => cy = attribute.length()?,
                b"rx" => rx = attribute.length()?,
                b"ry" => ry = attribute.length()?,
                _ => (),
            };
        }

        Ok(Self {
            style,
            center: [cx, cy].into(),
            radius: [rx, ry].into(),
        })
    }
}

#[derive(Debug)]
pub struct Image {
    pub style: Style,
    pub position: Vector2D<f64>,
    pub dimension: Vector2D<f64>,
    pub texture: Texture,
}

#[derive(Debug)]
pub struct Group {
    pub style: Style,
    pub elements: Vec<Element>,
}

impl Group {
    fn from_bytes_start(bytes: BytesStart, parent_style: Style) -> Result<Self, ReadError> {
        let style = Style::from_attributes(bytes.attributes(), parent_style)?;

        Ok(Self {
            style,
            elements: Vec::new(),
        })
    }
}

#[derive(Debug)]
pub struct SVG {
    pub dimension: Vector2D<f64>,
    pub elements: Vec<Element>,
}

impl SVG {
    fn from_bytes_start(bytes: BytesStart) -> Result<Self, ReadError> {
        let mut width = 300.0;
        let mut height = 150.0;

        for attribute in bytes.attributes() {
            let attribute = Attribute::parse(attribute?)?;
            match attribute.key {
                b"height" => height = attribute.length()?,
                b"width" => width = attribute.length()?,
                _ => (),
            };
        }

        Ok(Self {
            dimension: [width, height].into(),
            elements: Vec::new(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Style {
    pub stroke_color: Color,
    pub fill_color: Color,
    pub stroke_width: f64,
    pub miter_limit: f64,
    pub transform: Matrix3x3<f64>,
}

impl Style {
    const COLOR_NONE: Color = Color::RGBA(0, 0, 0, 0);

    pub const DEFAULT: Self = Self {
        stroke_color: Self::COLOR_NONE,
        fill_color: Self::COLOR_NONE,
        stroke_width: 1.0,
        miter_limit: 4.0,
        transform: Matrix3x3::IDENTITY3X3,
    };

    fn from_attributes(
        attributes: quick_xml::events::attributes::Attributes,
        mut parent_style: Style,
    ) -> Result<Self, ReadError> {
        const FLOAT_TO_8BIT: f64 = core::u8::MAX as f64;
        for attribute in attributes {
            let attribute = Attribute::parse(attribute?)?;
            match attribute.key {
                b"fill" => parent_style.fill_color = attribute.color(),
                b"fill-opacity" => {
                    parent_style.fill_color.a = (attribute.number()? * FLOAT_TO_8BIT) as u8
                }
                b"stroke" => parent_style.stroke_color = attribute.color(),
                b"stroke-opacity" => {
                    parent_style.stroke_color.a = (attribute.number()? * FLOAT_TO_8BIT) as u8
                }
                b"stroke-width" => parent_style.stroke_width = attribute.number()?,
                b"stroke-miterlimit" => parent_style.miter_limit = attribute.number()?,
                b"transform" => parent_style.transform *= attribute.transform_list()?,
                _ => (),
            };
        }

        Ok(parent_style)
    }
}

fn read_next_event(
    reader: &mut NsReader<BufReader<File>>,
    style_lifo: &mut Vec<Style>,
) -> Result<Element, EventStatus> {
    let parent_style = match style_lifo.last() {
        None => &Style::DEFAULT,
        Some(style) => style,
    };

    let mut buf = Vec::new();
    let next_event = reader.read_event_into(&mut buf)?;
    match next_event {
        Event::Start(start_tag_bytes) => {
            let (tag, style) =
                StartTag::from_start_tag_bytes(start_tag_bytes, (*parent_style).clone())?;

            style_lifo.push(style);

            Ok(Element::StartTag(tag))
        }
        // Event::Text(event) => unimplemented!(),
        Event::End(end_tag_bytes) => {
            if style_lifo.pop().is_none() {
                return Err(EventStatus::Error(ReadError::EndTagBeforeStart));
            }
            Ok(Element::EndTag(EndTag::from_end_tag_bytes(end_tag_bytes)?))
        }
        Event::Empty(empty_tag_bytes) => Ok(Element::EmptyTag(EmptyTag::from_empty_tag_bytes(
            empty_tag_bytes,
            (*parent_style).clone(),
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

    let mut style_lifo = Vec::new();
    let mut tag_lifo = Vec::new();

    loop {
        match read_next_event(&mut reader, &mut style_lifo) {
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
