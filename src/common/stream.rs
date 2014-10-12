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
#[allow(unused_imports)]
use super::{MIN_ALLOWED_LINE_SIZE};

pub static LINE_TOO_LONG: &'static str = "line too long";
pub static DATA_TOO_LONG: &'static str = "message too long";

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
///     MIN_ALLOWED_LINE_SIZE,
/// };
///
/// let mut smtp = SmtpStream::new(
///     TcpStream::connect("127.0.0.1", 25).unwrap(),
///     MIN_ALLOWED_LINE_SIZE,
///     false
/// );
///
/// println!("{}", smtp.read_line().unwrap());
/// ```
pub struct SmtpStream<S> {
    /// Underlying stream
    stream: S,
    /// Must be at least 1001 per RFC 5321, 1000 chars + 1 for transparency
    /// mechanism.
    max_line_size: uint,
    /// Buffer to make reading more efficient and allow pipelining
    buf: Vec<u8>,
    /// If `true`, will print debug messages of input and output to the console.
    debug: bool,
    /// The position of the `<CRLF>` found at the previous `read_line`.
    last_crlf: Option<uint>
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
    pub fn new(inner: S, max_line_size: uint, debug: bool) -> SmtpStream<S> {
        SmtpStream {
            stream: inner,
            max_line_size: max_line_size,
            // TODO: make line reading work even with a buffer smaller than the maximum line size.
            // Currently, this will not work because we only fill the buffer once per line, assuming
            // that the buffer is large enough.
            buf: Vec::with_capacity(max_line_size),
            debug: debug,
            last_crlf: None
        }
    }

    /// Remove the previous line from the buffer when reading a new line.
    fn move_buf(&mut self) {
        // Remove the last line, since we've used it already by now.
        match self.last_crlf {
            Some(p) => {
                // TODO: This could probably be optimised by shifting bytes instead
                // of re-allocating.
                self.buf = self.buf.as_slice().slice_from(p + 2).to_vec();
                self.buf.reserve(self.max_line_size);
            },
            _ => {}
        }

        self.last_crlf = None;
    }

    /// Fill the buffer to its limit.
    fn fill_buf(&mut self) -> IoResult<uint> {
        let len = self.buf.len();
        let cap = self.buf.capacity();

        // Read as much data as the buffer can hold without re-allocation.
        let res = self.stream.push(cap - len, &mut self.buf);

        res
    }

    /// Read an SMTP command. Ends with `<CRLF>`.
    pub fn read_line(&mut self) -> IoResult<&[u8]> {
        // Remove the previous line from the buffer before reading a new one.
        self.move_buf();

        match position_crlf(self.buf.as_slice()) {
            // First, let's check if the buffer already contains a line. This
            // reduces the number of syscalls.
            Some(last_crlf) => {
                let s = self.buf.slice_to(last_crlf);
                if self.debug {
                    println!("rsmtp: imsg: {}", s);
                }
                self.last_crlf = Some(last_crlf);
                Ok(s)
            },
            // If we don't have a line in the buffer, we'll read more input
            // and try again.
            None => {
                match self.fill_buf() {
                    Ok(_) => {
                        match position_crlf(self.buf.as_slice()) {
                            Some(last_crlf) => {
                                let s = self.buf.slice_to(last_crlf);
                                if self.debug {
                                    println!("rsmtp: imsg: {}", s);
                                }
                                self.last_crlf = Some(last_crlf);
                                Ok(s)
                            },
                            None => {
                                // If we didn't find a line, it means we had
                                // no `<CRLF>` in the buffer, which means that
                                // the line is too long.
                                Err(IoError {
                                    kind: InvalidInput,
                                    desc: LINE_TOO_LONG,
                                    detail: None
                                })                                
                            }
                        }
                    },
                    Err(err) => {
                        Err(err)
                    }
                }                
            }
        }

    }

    /// Write a line ended with `<CRLF>`.
    pub fn write_line(&mut self, s: &str) -> IoResult<()> {
        if self.debug {
            println!("rsmtp: omsg: {}", s);
        }
        // We use `format!()` instead of 2 calls to `write_str()` to reduce
        // the amount of syscalls and to send the string as a single packet.
        // I'm not sure if this is the right way to go though. If you think
        // this is wrong, please open a issue on Github.
        self.stream.write_str(format!("{}\r\n", s).as_slice())
    }
}

#[test]
fn test_new() {
    // This method is already tested via `test_read_line()`.
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
        stream = SmtpStream::new(file_write, MIN_ALLOWED_LINE_SIZE, false);
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
    stream = SmtpStream::new(file, 3, false);
    match stream.read_line() {
        Ok(_) => fail!(),
        Err(err) => {
            assert_eq!("line too long", err.desc);
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
    stream = SmtpStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line2");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line3");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/1line1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/1line2");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/2lines1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "bye bye world!");
    assert!(!stream.read_line().is_ok());

    expected = String::from_char(62, 'x');
    path = Path::new("tests/stream/xlines1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, MIN_ALLOWED_LINE_SIZE, false);
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
