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

use std::io::{Reader, Writer, IoError};
use std::vec::{Vec};
#[allow(unused_imports)]
use std::io::{Truncate, Open, Read, Write};
#[allow(unused_imports)]
use std::io::fs::{File};

/// The maximum line size as specified by RFC 5321.
static MAX_LINE_SIZE: uint = 512;

#[test]
fn test_static_vars() {
    assert_eq!(512, MAX_LINE_SIZE);
}

/// A stream specially made for reading SMTP commands.
///
/// It reads lines of input delimited by the &lt;CRLF&gt; sequence and with a maximum
/// size of 512 bytes, including the command word and the &lt;CRLF&gt; sequence. If
/// the input is not UTF8, non-UTF8 characters are replaced with `U+FFFD
/// REPLACEMENT CHARACTER` but no error is returned.
///
/// Returns `InvalidInput` if no line is found within 512 bytes of input.
///
/// # Example
/// ```no_run
/// use std::io::TcpStream;
/// use rsmtp::stream::SmtpStream;
///
/// let mut smtp = SmtpStream::new(TcpStream::connect("127.0.0.1", 2525).unwrap(), 65536);
///
/// println!("{}", smtp.read_line().unwrap());
/// ```
pub struct SmtpStream<S> {
    stream: S,
    /// The maximum message size, including headers and ending sequence.
    max_message_size: uint
}

#[deriving(Show, Eq, PartialEq)]
pub enum SmtpStreamError {
    ReadFailed(IoError),
    WriteFailed(IoError),
    LineTooLong,
    TooMuchData
}

impl<S> SmtpStream<S> {
    /// Create a new `SmtpStream` from another stream.
    pub fn new(inner: S, max_message_size: uint) -> SmtpStream<S> {
        SmtpStream {
            stream: inner,
            max_message_size: max_message_size
        }
    }
}

impl<R: Reader> SmtpStream<R> {
    /// Read the data section of an email. Ends with "&lt;CRLF&gt;.&lt;CRLF&gt;".
    pub fn read_data(&mut self) -> Result<Vec<u8>, SmtpStreamError> {
        let mut data: Vec<u8> = Vec::with_capacity(512);
        let end = [13u8, 10u8, 46u8, 13u8, 10u8]; // CRLF.CRLF
        let end_len = end.len();
        let mut too_long = false;
        let mut last_5: Vec<u8> = vec!(0u8, 0u8, 0u8, 0u8, 0u8);

        loop {
            // If we have previously read as much data as possible and still are not finished
            // reading, stop here.
            if data.len() >= self.max_message_size && !too_long {
                too_long = true;
            }

            // Try to get more data and see if we have got it all.
            let byte_res = self.stream.read_byte();
            match byte_res {
                Ok(b) => {
                    // Only keep remaining data if we are allowed too. Otherwise, discard it too
                    // avoid out of memory errors.
                    if data.len() < self.max_message_size {
                        data.push(b);
                    }
                    // Since we always have 5 bytes in here, this should never fail.
                    last_5.remove(0).unwrap();
                    last_5.push(b);

                    // Let's see if we have read all the data.
                    let data_len = data.len();
                    if data_len >= end_len && data.slice_from(data_len - end_len) == end {
                        data.truncate(data_len - end_len);
                        break;
                    }
                },
                Err(err) => {
                    return Err(ReadFailed(err))
                }
            }
        }
        if too_long {
            Err(TooMuchData)
        } else {
            Ok(data)
        }
    }

    /// Read one line of input.
    pub fn read_line(&mut self) -> Result<Vec<u8>, SmtpStreamError> {
        let mut data: Vec<u8> = Vec::with_capacity(MAX_LINE_SIZE);
        let end = [13u8, 10u8]; // CRLF
        let end_len = end.len();
        let mut too_long = false;
        let mut last_2: Vec<u8> = vec!(0u8, 0u8);

        loop {
            // If we have previously read as much data as possible and still are not finished
            // reading, stop here.
            if data.len() >= MAX_LINE_SIZE && !too_long {
                too_long = true;
            }

            // Try to get more data and see if we have got it all.
            let byte_res = self.stream.read_byte();
            match byte_res {
                Ok(b) => {
                    // Only keep remaining data if we are allowed too. Otherwise, discard it too
                    // avoid out of memory errors.
                    if data.len() < self.max_message_size {
                        data.push(b);
                    }
                    // Since we always have 2 bytes in here, this should never fail.
                    last_2.remove(0).unwrap();
                    last_2.push(b);

                    // Let's see if we have read all the line
                    let data_len = data.len();
                    if data_len >= end_len && data.slice_from(data_len - end_len) == end {
                        data.truncate(data_len - end_len);
                        break;
                    }
                },
                Err(err) => return Err(ReadFailed(err))
            }
        }
        if too_long {
            Err(LineTooLong)
        } else {
            Ok(data)
        }
    }

}

impl<W: Writer> SmtpStream<W> {
    /// Write a line ended with &lt;CRLF&gt;.
    pub fn write_line(&mut self, s: &str) -> Result<(), SmtpStreamError> {
        match self.stream.write_str(format!("{}\r\n", s).as_slice()) {
            Ok(_) => Ok(()),
            Err(err) => Err(WriteFailed(err))
        }
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

    path = Path::new("tests/stream/1line2");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, 65536);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/2lines1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, 65536);
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "hello world!");
    assert_eq!(String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string().as_slice(), "bye bye world!");
    assert!(!stream.read_line().is_ok());

    expected = String::from_char(62, 'x');
    path = Path::new("tests/stream/xlines1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file, 65536);
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
