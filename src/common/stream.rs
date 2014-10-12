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

use std::io::{Reader, Writer, IoResult, IoError, InvalidInput, EndOfFile};
use std::vec::Vec;
#[allow(unused_imports)]
use std::io::{Truncate, Open, Read, Write};
#[allow(unused_imports)]
use std::io::fs::File;
#[allow(unused_imports)]
use super::{MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE};

static LINE_TOO_LONG: &'static str = "line too long";
static DATA_TOO_LONG: &'static str = "message too long";

#[test]
fn test_static_vars() {
    // Already tested in the limits test further down.
}

/// A stream specially made for reading SMTP commands, messages and writing replies.
///
/// # Example
/// ```no_run
/// use std::io::TcpStream;
/// use rsmtp::common::stream::SmtpStream;
/// use rsmtp::common::{
///     MIN_ALLOWED_MESSAGE_SIZE,
///     MIN_ALLOWED_LINE_SIZE
/// };
///
/// let mut smtp = SmtpStream::new(
///     TcpStream::connect("127.0.0.1", 25).unwrap(),
///     MIN_ALLOWED_MESSAGE_SIZE,
///     MIN_ALLOWED_LINE_SIZE
/// );
///
/// println!("{}", smtp.read_line().unwrap());
/// ```
pub struct SmtpStream<S> {
    /// Underlying stream
    stream: S,
    /// The maximum message size, including headers and ending sequence.
    max_message_size: uint,
    /// The maximum message size.
    ///
    /// Must be at least 1001 per RFC 5321, 1000 chars + 1 for transparency
    /// mechanism.
    max_line_size: uint,
    /// Buffer to make reading more efficient and allow pipelining
    buf: Vec<u8>
}

// The state of the `<CRLF>` search inside a buffer. See below.
enum CRLFState {
    // We are looking for `<CR>`.
    Cr,
    // We are looking for `<LF>`.
    Lf
}

// Find the position of the first `<CRLF>` in a buffer.
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
                    // Subtract 1 to account for the \r, seen previously.
                    return Some(index - 1);
                }
            },
        }
        index += 1;
    }

    None
}

impl<S: Reader+Writer> SmtpStream<S> {
    /// Create a new `SmtpStream` from another stream.
    pub fn new(inner: S, max_message_size: uint, max_line_size: uint) -> SmtpStream<S> {
        SmtpStream {
            stream: inner,
            max_message_size: max_message_size,
            max_line_size: max_line_size,
            // TODO: make line reading work even with a buffer smaller than the maximum line size.
            // Currently, this will not work because we only fill the buffer once per line, assuming
            // that the buffer is large enough.
            buf: Vec::with_capacity(max_line_size)
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
        // First of all, let's see if our buffer has what we need. Maybe it's
        // that easy :-)
        match self.find_line() {
            Ok(line) => Ok(line),
            Err(_) => {
                // Try to fill the buffer in the hope we get a line.
                match self.fill_buf() {
                    Err(err) => {
                        // It could be the case, that we've already read everything but
                        // still have a line left in the buffer, so we need to check if
                        // that's the case if we get EndOfFile.
                        match err.kind {
                            EndOfFile => self.find_line(),
                            _ => Err(err)
                        }
                    },
                    // Here, we've read some data, so let's try to find a line.
                    Ok(_) => {
                        self.find_line()
                    }
                }
            }
        }
    }

    fn find_line(&mut self) -> IoResult<Vec<u8>> {
        match position_crlf(self.buf.as_slice()) {
            Some(p) => {
                // TODO: This could probably be optimised to use one less alloc, no?
                let line = self.buf.as_slice().slice_to(p).into_vec();
                self.buf = self.buf.as_slice().slice_from(p + 2).into_vec();
                self.buf.reserve(self.max_line_size);
                Ok(line)
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

    /// Read the email body after a DATA command. Ends with `<CRLF>.<CRLF>`.
    pub fn read_data(&mut self) -> IoResult<Vec<u8>> {
        let mut data = Vec::with_capacity(2048);

        loop {
            match self.read_line() {
                Err(err) => {
                    return Err(err)
                },
                Ok(line) => {
                    // Here, we check that we have already got some data, which
                    // means that we have read a line, which means we have just
                    // seen `<CRLF>`. And then, we check if the current line
                    // which we know to end with `<CRLF>` as well contains a
                    // single dot.
                    // All in all, this means we check for `<CRLF>.<CRLF>`.
                    if data.len() != 0 && line.as_slice() == &['.' as u8] {
                        break;
                    }
                    // TODO: support transparency.

                    data.extend(line.into_iter());
                    if data.len() > self.max_message_size {
                        return Err(IoError {
                            kind: InvalidInput,
                            desc: DATA_TOO_LONG,
                            detail: None
                        })
                    }
                }
            }
        }

        Ok(data)
    }

    /// Write a line ended with `<CRLF>`.
    pub fn write_line(&mut self, s: &str) -> IoResult<()> {
        self.stream.write_str(format!("{}\r\n", s).as_slice())
    }
}

#[test]
fn test_new() {
    // This method is already tested via `test_read_line()`.
}

#[test]
fn test_read_data_ok() {
    let mut path: Path;
    let mut file: File;
    let mut stream: SmtpStream<File>;
    let mut expected: String;

    path = Path::new("tests/stream/data_ok");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
    expected = String::from_utf8_lossy(stream.read_data().unwrap().as_slice()).into_string();
    assert_eq!("Hello world!\nBlabla\n", expected.as_slice());
}

#[test]
fn test_read_data_not_ok() {
    let mut path: Path;
    let mut file: File;
    let mut stream: SmtpStream<File>;

    path = Path::new("tests/stream/data_not_ok");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
    assert!(!stream.read_data().is_ok());
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
        stream = SmtpStream::new(file_write, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
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
fn test_limits() {
    let mut path: Path;
    let mut file: File;
    let mut stream: SmtpStream<File>;

    path = Path::new("tests/stream/1line1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, 3);
    match stream.read_line() {
        Ok(_) => fail!(),
        Err(err) => {
            assert_eq!("line too long", err.desc);
            assert_eq!(InvalidInput, err.kind);
        }
    }

    path = Path::new("tests/stream/1line1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, 3, MIN_ALLOWED_LINE_SIZE);
    match stream.read_data() {
        Ok(_) => fail!(),
        Err(err) => {
            assert_eq!("message too long", err.desc);
            assert_eq!(InvalidInput, err.kind);
        }
    }
}

#[test]
fn test_read_line() {
    let mut path: Path;
    let mut file: File;
    let mut stream: SmtpStream<File>;
    let mut expected: String;

    path = Path::new("tests/stream/0line1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line2");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line3");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/1line1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/1line2");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/2lines1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "bye bye world!");
    assert!(!stream.read_line().is_ok());

    expected = String::from_char(62, 'x');
    path = Path::new("tests/stream/xlines1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_MESSAGE_SIZE, MIN_ALLOWED_LINE_SIZE);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string(), expected);
    assert!(!stream.read_line().is_ok());
}
