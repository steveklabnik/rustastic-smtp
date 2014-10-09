// Copyright 2014 The Rustastic SMTP Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tools for reading/writing from SMTP clients to SMTP servers and vice-versa.

use std::io::{Reader, Writer, IoResult, IoError, InvalidInput};
use std::vec::Vec;
#[allow(unused_imports)]
use std::io::{Truncate, Open, Read, Write};
#[allow(unused_imports)]
use std::io::fs::File;
use std::mem;

/// The maximum line size as specified by RFC 5321.
static MAX_LINE_SIZE: uint = 512;
static LINE_TOO_LONG: &'static str = "line too long";

#[test]
fn test_static_vars() {
    assert_eq!(512, MAX_LINE_SIZE);
}

/// A stream specially made for reading SMTP commands, messages and writing replies.
///
/// # Example
/// ```no_run
/// use std::io::TcpStream;
/// use rsmtp::common::stream::SmtpStream;
///
/// let mut smtp = SmtpStream::new(TcpStream::connect("127.0.0.1", 2525).unwrap(), 65536);
///
/// println!("{}", smtp.read_line().unwrap());
/// ```
pub struct SmtpStream<S> {
    /// Underlying stream
    stream: S,
    /// The maximum message size, including headers and ending sequence.
    max_message_size: uint,
    /// Buffer to make reading more efficient and allow pipelining
    buf: Vec<u8>
}

enum EndOfMessageState {
    Cr1,
    Lf1,
    Dot,
    Cr2,
    Lf2
}

enum CRLFState {
    Cr,
    Lf
}

fn position_crlf(buf: &[u8]) -> Option<uint> {
    let mut state = Cr;
    let mut index = 0;

    for byte in buf.iter() {
        match state {
            Cr => {
                if byte == &13 {
                    state = Lf;
                }
            },
            Lf => {
                if byte == &10 {
                    return Some(index);
                }
            },
        }
        index += 1;
    }
    None
}

fn position_eom(buf: &[u8]) -> Option<uint> {
    let mut state = Cr1;
    let mut index = 0;

    for byte in buf.iter() {
        match state {
            Cr1 => {
                if byte == &13 {
                    state = Lf1;
                }
            },
            Lf1 => {
                if byte == &10 {
                    state = Dot;
                }
            },
            Dot => {
                if byte == &('.' as u8) {
                    state = Cr2;
                }
            },
            Cr2 => {
                if byte == &13 {
                    state = Lf2;
                }
            },
            Lf2 => {
                if byte == &10 {
                    return Some(index);
                }
            },
        }
        index += 1;
    }
    None
}

impl<S: Reader+Writer> SmtpStream<S> {
    /// Create a new `SmtpStream` from another stream.
    pub fn new(inner: S, max_message_size: uint) -> SmtpStream<S> {
        SmtpStream {
            stream: inner,
            max_message_size: max_message_size,
            buf: Vec::with_capacity(MAX_LINE_SIZE)
        }
    }

    fn fill_buf(&mut self) -> IoResult<uint> {
        let len = self.buf.len();
        let cap = self.buf.capacity();

        // Read as much data as the buffer can hold without re-allocation.
        match self.stream.push(cap - len, &mut self.buf) {
            Err(err) => {
                Err(err)
            },
            Ok(data) => {
                Ok(data)
            }
        }
    }

    /// Read an SMTP command. Ends with `<CRLF>`.
    pub fn read_line(&mut self) -> IoResult<Vec<u8>> {
        
        // Try to fill the buffer in the hope we get a line.
        match self.fill_buf() {
            Err(err) => {
                Err(err)
            },
            // Here, we've read some data, so let's try to find a line.
            Ok(_) => {
                match position_crlf(self.buf.as_slice()) {
                    Some(p) => {
                        // Keep the rest of the line, so what's after `<CRLF>`.
                        let mut new_buf = self.buf.as_slice().slice_from(p + 2).into_vec();
                        new_buf.reserve(MAX_LINE_SIZE);

                        // For our returned line, remove what's after `<CRLF>`.
                        self.buf.truncate(p);

                        // We'll save some information about our line, which will
                        // allow us to return it without re-allocating memory.
                        let len = self.buf.len();
                        let cap = self.buf.capacity();
                        let ptr = self.buf.as_mut_ptr();

                        // Finally, return our line without re-allocation.
                        Ok(unsafe {
                            mem::forget(&self.buf);
                            // Put the rest of the old Stream's buffer back where it belongs.
                            self.set_buf(new_buf);
                            Vec::from_raw_parts(len, cap, ptr)
                        })
                    }
                    None => {
                        Err(IoError {
                            kind: InvalidInput,
                            desc: LINE_TOO_LONG,
                            detail: None
                        })
                    }
                }
            }
        }
    }

    /// Read the email body after a DATA command. Ends with `<CRLF>.<CRLF>`.
    pub fn read_data(&mut self) -> IoResult<Vec<u8>> {
        Ok(Vec::new())
    }

    /// Write a line ended with `<CRLF>`.
    pub fn write_line(&mut self, s: &str) -> IoResult<()> {
        self.stream.write_str(format!("{}\r\n", s).as_slice())
    }
}

#[test]
fn test_new() {
    // Testing via `test_read_line()`.
}

#[test]
fn test_read_data() {
    let mut path: Path;
    let mut file: File;
    let mut stream: SmtpStream<File>;
    let mut expected: String;

    path = Path::new("tests/stream/data_ok");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, 65536);
    expected = String::from_utf8_lossy(stream.read_data().unwrap().as_slice()).into_string();
    assert_eq!("Hello world!\nBlabla\n", expected.as_slice());
}

#[test]
fn test_write_line() {
    // Use a block so the file gets closed at the end of it.
    {
        let mut path_write: Path;
        let mut file_write: File;
        let mut stream: SmtpStream<File>;

        path_write = Path::new("tests/stream/write_line");
        file_write = File::open_mode(&path_write, Truncate, Write).unwrap();
        stream = SmtpStream::new(file_write, 65536);
        stream.write_line("HelloWorld").unwrap();
        stream.write_line("ByeBye").unwrap();
    }
    let mut path_read: Path;
    let mut file_read: File;
    let mut expected: String;

    path_read = Path::new("tests/stream/write_line");
    file_read = File::open_mode(&path_read, Open, Read).unwrap();
    expected = file_read.read_to_string().unwrap();
    assert_eq!("HelloWorld\r\nByeBye\r\n", expected.as_slice());
}

#[test]
fn test_read_line() {
    let mut path: Path;
    let mut file: File;
    let mut stream: SmtpStream<File>;
    let mut expected: String;

    path = Path::new("tests/stream/0line1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, 65536);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line2");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, 65536);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line3");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, 65536);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/1line1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, 65536);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    // path = Path::new("tests/stream/1line2");
    // file = File::open(&path).unwrap();
    // stream = SmtpStream::new(file, 65536);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    // assert!(!stream.read_line().is_ok());

    // path = Path::new("tests/stream/2lines1");
    // file = File::open(&path).unwrap();
    // stream = SmtpStream::new(file, 65536);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "bye bye world!");
    // assert!(!stream.read_line().is_ok());

    // expected = String::from_char(62, 'x');
    // path = Path::new("tests/stream/xlines1");
    // file = File::open(&path).unwrap();
    // stream = SmtpStream::new(file, 65536);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    // assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    // assert!(!stream.read_line().is_ok());
}
