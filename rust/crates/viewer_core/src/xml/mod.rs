use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::ViewerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlElement {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<XmlElement>,
    pub text: String,
}

impl XmlElement {
    pub fn local_name(&self) -> &str {
        self.name.rsplit(':').next().unwrap_or(&self.name)
    }

    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name || key.rsplit(':').next() == Some(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn required_attribute(&self, name: &str) -> Result<&str, ViewerError> {
        self.attribute(name).ok_or(ViewerError::InvalidDocument)
    }

    pub fn child(&self, local_name: &str) -> Option<&XmlElement> {
        self.children
            .iter()
            .find(|child| child.local_name() == local_name)
    }
}

pub fn parse_document(xml: &str) -> Result<XmlElement, ViewerError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut stack: Vec<XmlElement> = Vec::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(start)) => stack.push(new_element(&reader, &start)?),
            Ok(Event::Empty(start)) => {
                let element = new_element(&reader, &start)?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(element);
                } else {
                    return Ok(element);
                }
            }
            Ok(Event::Text(text)) => {
                if let Some(current) = stack.last_mut() {
                    let unescaped = text.decode().map_err(|_| ViewerError::InvalidDocument)?;
                    if !unescaped.is_empty() {
                        current.text.push_str(&unescaped);
                    }
                }
            }
            Ok(Event::End(_)) => {
                let element = stack.pop().ok_or(ViewerError::InvalidDocument)?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(element);
                } else {
                    return Ok(element);
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(_) => return Err(ViewerError::InvalidDocument),
        }
    }

    Err(ViewerError::InvalidDocument)
}

fn new_element(reader: &Reader<&[u8]>, start: &BytesStart<'_>) -> Result<XmlElement, ViewerError> {
    let name = reader
        .decoder()
        .decode(start.name().as_ref())
        .map_err(|_| ViewerError::InvalidDocument)?
        .into_owned();

    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|_| ViewerError::InvalidDocument)?;
        let key = reader
            .decoder()
            .decode(attribute.key.as_ref())
            .map_err(|_| ViewerError::InvalidDocument)?
            .into_owned();
        let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|_| ViewerError::InvalidDocument)?
            .into_owned();
        attributes.push((key, value));
    }

    Ok(XmlElement {
        name,
        attributes,
        children: Vec::new(),
        text: String::new(),
    })
}
