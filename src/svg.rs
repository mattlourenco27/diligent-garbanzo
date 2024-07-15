use std::{collections::HashMap, os::raw, path::Path};

use quick_xml::{events::{attributes, Event}, NsReader};

use crate::color::Color;

#[derive(Debug)]
pub enum Error {
    XMLError(quick_xml::errors::Error),
}

impl From<quick_xml::errors::Error> for Error {
    fn from(value: quick_xml::errors::Error) -> Self {
        return Self::XMLError(value);
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::XMLError(err) => write!(f, "XML Error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::XMLError(err) => Some(err),
        }
    }
}

pub enum SVGElementType {
    POINT,
    LINE,
    POLYLINE,
    RECT,
    POLYGON,
    ELLIPSE,
    IMAGE,
    GROUP,
}

pub struct Style {
    stroke_color: Color,
    fill_color: Color,
    stroke_width: f32,
    miter_limit: f32,
}

pub struct SVG<'a> {
    raw: Vec<u8>,
    pub local_name: &'a [u8],
    pub attributes: HashMap<&'a [u8], &'a [u8]>,
}

pub fn read_from_file(path: &Path) -> Result<(), Error> {
    let mut reader = NsReader::from_file(path)?;

    let mut buf = Vec::new();
    let mut txt = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                dbg!(&e);
                let local = e.local_name();
                dbg!(local);
                let attributes = e.attributes();
                for attribute in attributes {
                    let attribute = match attribute {
                        Ok(attribute) => attribute,
                        Err(err) => return Err(Error::from(quick_xml::errors::Error::from(err))),
                    };
                    let local = attribute.key.local_name();
                    dbg!(local);
                }
            }
            Event::Text(e) => txt.push(e.unescape()?.into_owned()),
            Event::End(e) => {}
            Event::Empty(e) => {
                let local = e.local_name();
                let attributes = e.attributes();
                for attribute in attributes {
                    let attribute = match attribute {
                        Ok(attribute) => attribute,
                        Err(err) => return Err(Error::from(quick_xml::errors::Error::from(err))),
                    };
                    if attribute.key.local_name().into_inner() == b"fill" {
                        let raw_color = [50, 50, 50, 50];
                    }
                }
            }
            Event::Eof => break,
            _ => (),
        }
        buf.clear();
    }

    Ok(())
}
