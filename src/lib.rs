//! # crumble
//!
//! A robust, minimal library for parsing MIME documents.
//!
//! Supports UTF-8, multipart documents, and nested documents. `crumble` assumes input is mostly
//! compliant and tries to parse input as best as possible.
//!
//! Output is a minimal AST-like structure, with no filtering. This structure should be further
//! processed to produce useful work. See for example [`crinkle`][1]. 
//!
//! # Example
//!
//! ```
//! use crumble::Message;
//! let message = load_mime_file(); // Example! The consumer must have some source of documents
//! let message = Message::new(&message)?; // Handle errors
//! ```
//! [1]: https://git.sr.ht/~happy_shredder/crinkle

#![feature(test)]
#[cfg(test)] mod tests;
#[macro_use] extern crate lazy_static;

use regex::Regex;
use std::fmt::Write;

/// Internal error type.
///
/// # Remarks
/// This library is permissive and tries to parse input as best as it can.
/// Hence very few errors are possible. Trying to parse non-MIME documents is undefined behaviour.
#[derive(Debug,PartialEq)]
pub enum Error {
    Unknown,
    InvalidString,
    ParseError,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unknown => write!(f, "Error parsing message: Unknown error"),
            Error::InvalidString => write!(f, "Error parsing message: Invalid string"),
            Error::ParseError => write!(f, "Error parsing message: Invalid document"),
        }
    }
}

/// Wraps a String tuple for more literate usage and application of traits.
#[derive(Debug,PartialEq)]
pub struct Header {
    pub key: String,
    pub value: String,
}

impl Header {
    pub fn new(key: &str, value: &str) -> Header {
        Header {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

impl std::string::ToString for Header {
    fn to_string(&self) -> String {
        format!("{}: {}", self.key, self.value)
    }
}

// Wrapper to avoid error.
trait ToString {
    fn to_string(&self) -> String;
}

impl ToString for Vec<Header> {
    fn to_string(&self) -> String {
        let tmp: Vec<String> = self.iter().map(|x| x.to_string()).collect();
        let tmp = tmp.join("\n");
        String::from(tmp)
    }
}

/// Representation of a section of a MIME document.
///
/// MIME sections can be some text; a header and some text or data; or nested combinations.
/// This parser may produce empty sections.
#[derive(Debug, PartialEq)]
pub enum Section {
    Plain {body: Vec<u8>},
    Multipart {
        headers: Vec<Header>,
        body: Vec<Box<Section>>,
    },
    Empty,
}

impl std::string::ToString for Section {
    fn to_string(&self) -> String {
        let mut section_string = String::new();
        match self {
            Section::Plain {body} => write!(&mut section_string, "{}", std::str::from_utf8(body).unwrap()).expect("Error constructing string."),
            Section::Multipart {headers, body} => {
                write!(&mut section_string, "{}\n------------\n", headers.to_string()).expect("Error constructing string.");
                for section in body {
                    write!(&mut section_string, "\n{}\n", section.to_string()).expect("Error constructing string.");
                }
                write!(&mut section_string, "------------").expect("Error constructing string.");
            },
            Section::Empty => (),
        }
        section_string
    }
}

impl Section {
    fn new(raw_section: &str) -> Result<Section, Box<dyn std::error::Error + 'static>> {
        // A section can either be just some plain text, or be split into headers/body.
        // That body is also section.
        // If the raw section has no headers, return it as plain
        // If it has headers, split off the headers and recurse

        // Catch leftover from multipart parsing
        if raw_section == "--\n" {
            return Ok(Section::Empty);
        }

        // A section with headers has a different parsing pipeline than one without.
        if Section::has_headers(raw_section)? {
            Section::parse_multipart(raw_section)
        } else {
            Ok(Section::Plain {body: raw_section.as_bytes().to_vec()})
        }
    }

    fn has_headers(raw_message: &str) -> Result<bool, Box<dyn std::error::Error + 'static>> {
        // If there are headers there should be a content-type
        // Note that headers may be separated by a boundary (nested sections) or newlines (not
        // nested)
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(Content-Type|Content-type|content-type): .+?").unwrap();
        }
        // Performance: Assume that the header is not too long and the boundary appears early
        if raw_message.len() > 3000 {
            Ok(RE.is_match(&raw_message[0..3000]))
        } else {
            Ok(RE.is_match(raw_message))
        }
    }

    fn has_boundary(raw_message: &str) -> Result<bool, Box<dyn std::error::Error + 'static>> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(boundary|Boundary)=.+?").unwrap();
        }
        // Performance: Assume that the header is not too long and the boundary appears early
        if raw_message.len() > 3000 {
            Ok(RE.is_match(&raw_message[0..3000]))
        } else {
            Ok(RE.is_match(raw_message))
        }
    }

    fn parse_multipart(raw_section: &str) -> Result<Section, Box<dyn std::error::Error + 'static>> {
        // The body can contain a series of (possibly nested) sections
        // So check for a boundary.
        // If there is a boundary, split the body and iterate.
        // Otherwise, just return a single-entry Vec

        if Section::has_boundary(raw_section)? {
            lazy_static! {
                static ref RE: Regex = Regex::new(r#"(boundary|Boundary)=("|')(?P<boundary>[[:print:]]+?)("|')"#).unwrap();
            }
            let boundary = match RE.captures(raw_section) {
                Some(c) => c["boundary"].to_string(),
                None => String::new(),
            };
            if boundary.len() == 0 {
                return Err(Box::new(Error::ParseError));
            }
            // Each section is separated by --<boundary>, and finishes with --<boundary>--
            let boundary = format!("--{}", boundary);
            let raw_sections: Vec<&str> = raw_section.split(boundary.as_str()).collect();

            let raw_headers = raw_sections[0];
            let headers = parse_headers(raw_headers)?;

            let mut sections = Vec::new();
            let raw_sections = &raw_sections[1..raw_sections.len() - 1]; // Drop empty section at tail

            for section in raw_sections {
                // Recursively construct sections
                let section = Section::new(&section)?;
                sections.push(Box::new(section));
            }

            Ok(Section::Multipart {
                headers: headers,
                body: sections,
            })
        } else {
            // Separate out headers
            lazy_static! {
                static ref RE: Regex = Regex::new(r"\n{2,}|\r{2,}").unwrap();
            }
            let split: Vec<&str> = RE.splitn(raw_section, 2).collect();
            let raw_headers = split[0];
            let headers = parse_headers(raw_headers)?;

            // Process body
            let body = split[1];
            let body = Section::new(&body)?;
            let sections = vec![Box::new(body)];

            Ok(Section::Multipart {
                headers: headers,
                body: sections,
            })
        }
    }
}

/// Representation of a MIME document.
///
/// MIME documents have a large initial key-value header, followed by one or more text/data sections.
/// A section can be some plain text; a header with text or data; or some nested combination.
#[derive(Debug)]
pub struct Message {
    pub headers: Vec<Header>,
    pub sections: Vec<Section>,
}

impl std::string::ToString for Message {
    fn to_string(&self) -> String {
        let mut message = String::new();
        write!(&mut message, "########################\n{}\n########################\n", self.headers.to_string()).expect("Error constructing string.");
        for section in &self.sections {
            write!(&mut message, "{}\n########################\n", section.to_string()).expect("Error constructing string.");
        }
        message
    }
}

impl Message {
    /// Parse a MIME document and return structured representation.
    /// Performance should be reasonable: provided tests take between 5 and 200μs per document.
    pub fn new(raw_message: &str) -> Result<Message, Box<dyn std::error::Error + 'static>> {
        // Multipart and plain messages require entirely different parsing pathways
        if Message::is_multipart(raw_message)? {
            Message::parse_multipart(raw_message)
        } else {
            Message::parse_plain(raw_message)
        }
    }

    fn is_multipart(raw_message: &str) -> Result<bool, Box<dyn std::error::Error + 'static>> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(Content-Type|Content-type|content-type): multipart.+?").unwrap();
        }
        Ok(RE.is_match(&raw_message))
    }

    fn parse_plain(raw_message: &str) -> Result<Message, Box<dyn std::error::Error + 'static>> {
        // Plain messages separate the headers from the body with more than 2 newlines
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\n{2,}|\r{2,}").unwrap();
        }
        let split: Vec<&str> = RE.splitn(raw_message, 2).collect();

        if split.len() != 2 || split[0].len() == 0 || split[1].len() == 0 {
            return Err(Box::new(Error::InvalidString));
        }

        let raw_headers = split[0];
        let headers = parse_headers(raw_headers)?;

        // Everything after the header is by definition the body. There is only one section.
        let tmp = split[1];
        let sections = vec![Section::new(&tmp)?];

        Ok(Message {
            headers: headers,
            sections: sections,
        })
    }

    fn parse_multipart(raw_message: &str) -> Result<Message, Box<dyn std::error::Error + 'static>> {
        // Multipart messages separate parts using a boundary string, defined in the main headers
        // Any reasonable string after a `boundary="` is the boundary
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"(Boundary|boundary)=("|')(?P<boundary>[[:print:]]+?)("|')"#).unwrap();
        }
        let b = match RE.captures(raw_message) {
            Some(c) => c["boundary"].to_string(),
            None => return Err(Box::new(Error::InvalidString)),
        };
        let boundary = format!("--{}", b);
        let raw_parts: Vec<&str> = raw_message.split(boundary.as_str()).collect();

        let raw_headers = raw_parts[0];
        let headers = parse_headers(raw_headers)?;

        let mut sections = Vec::new();
        let raw_parts = &raw_parts[1..raw_parts.len()];

        // Parse each section
        for section in raw_parts {
            let section = Section::new(section)?; // Note that this constructor will recursively build sections, as required
            sections.push(section);
        }

        Ok(Message {
            headers: headers,
            sections: sections,
        })
    }
}

// Find keys and values for each header
fn parse_headers(raw_headers: &str) -> Result<Vec<Header>, Box<dyn std::error::Error + 'static>> {
    // A MIME key is a string of letters|numbers|-|_, followed by a :
    // It starts on it's own line (i.e. after a \n)
    // While the spec requires a header to be all on its own line,
    // parsers in the wild (e.g. GMail) will split headers across mutliple lines.
    // Hence, a value is everything between two keys.
    // So, find the positions of all the keys -> extract key String
    // Then, infer the position of text between subsequent keys -> extract value String

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?m)^[0-9A-Za-z_\-]+:").unwrap();
    }
    let mut header_indices: Vec<(usize,usize)> = Vec::new();

    // Construct a vector of key positions
    for header in RE.find_iter(raw_headers) {
        header_indices.push((header.start(), header.end() - 1)); // Strip off :
    }

    // Convert key positions to key and value strings
    let mut headers: Vec<Header> = Vec::new();
    for (index, header) in header_indices.iter().enumerate() {
        let key = String::from(&raw_headers[header.0..header.1]);
        let key = key.trim();
        let key = String::from(key);
        let key = key.to_lowercase();

        // The final value is not between two keys: it is final key to end of string
        if index < header_indices.len() - 1 {
            let value = String::from(&raw_headers[header.1 + 2..header_indices[index + 1].0]);  // Correct for :
            let value = value.trim();
            headers.push(Header::new(&key, &value));
        } else {
            let value = String::from(&raw_headers[header.1 + 2..]);
            let value = value.trim();
            headers.push(Header::new(&key, &value));
        }
    }

    Ok(headers)
}
