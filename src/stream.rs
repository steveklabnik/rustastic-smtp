use std::io::{Reader, Writer, IoErrorKind, InvalidInput};
use std::string::{String};
#[allow(unused_imports)]
use std::io::{Truncate, Open, Read, Write};
#[allow(unused_imports)]
use std::io::fs::{File};

/// The maximum line size as specified by RFC 5321.
static MAX_LINE_SIZE: uint = 512;

/// Default minimum data allocation.
static DATA_ALLOC_SIZE: uint = 512;

#[test]
fn test_static_vars() {
    assert_eq!(512, MAX_LINE_SIZE);
    assert_eq!(512, DATA_ALLOC_SIZE);
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
/// let mut smtp = SmtpStream::new(TcpStream::connect("127.0.0.1", 2525).unwrap());
///
/// println!("{}", smtp.read_line().unwrap());
/// ```
pub struct SmtpStream<S> {
    stream: S,
    vec: Vec<u8>
}

impl<S> SmtpStream<S> {
    /// Create a new `SmtpStream` from another stream.
    pub fn new(inner: S) -> SmtpStream<S> {
        SmtpStream {
            stream: inner,
            vec: Vec::with_capacity(MAX_LINE_SIZE)
        }
    }
}

impl<R: Reader> SmtpStream<R> {
    /// Read the data section of an email. Ends with "&lt;CRLF&gt;.&lt;CRLF&gt;".
    pub fn read_data(&mut self) -> Result<String, IoErrorKind> {
        let mut data = String::with_capacity(DATA_ALLOC_SIZE);

        loop {
            let byte_res = self.stream.read_byte();
            match byte_res {
                Ok(b) => {
                    data = data.append(String::from_byte(b).as_slice());
                    if data.as_slice().ends_with("\r\n.\r\n") {
                        let len = data.len();
                        data.truncate(len - 5);
                        break;
                    }
                },
                Err(err) => {
                    return Err(err.kind)
                }
            }
        }
        Ok(data)
    }

    /// Read one line of input.
    pub fn read_line(&mut self) -> Result<String, IoErrorKind> {
        self.vec.clear();
        loop {
            // If we have previously read 512 bytes and have not found a line,
            // stop here.
            if self.vec.len() == 512 {
                return Err(InvalidInput)
            }

            // Try to read one more byte and see if a line is formed.
            let byte_res = self.stream.read_byte();
            match byte_res {
                Ok(b) => {
                    self.vec.push(b);
                    // A line ends with \r\n (or 13/10 in decimal), so we
                    // check for these bytes at the end of our buffer.
                    let len = self.vec.len();
                    if len >= 2 {
                        if self.vec[len - 2] == 13 && self.vec[len - 1] == 10 {
                            return Ok(String::from_utf8_lossy(
                                self.vec.slice_to(len - 2)
                            ).into_string());
                        }
                    }
                },
                Err(err) => return Err(err.kind)
            }
        }
    }

}

impl<W: Writer> SmtpStream<W> {
    /// Write a line ended with &lt;CRLF&gt;.
    pub fn write_line(&mut self, s: &str) -> Result<(), IoErrorKind> {
        match self.stream.write_str(format!("{}\r\n", s).as_slice()) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.kind)
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
    stream = SmtpStream::new(file);
    expected = stream.read_data().unwrap();
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
        stream = SmtpStream::new(file_write);
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
    stream = SmtpStream::new(file);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line2");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line3");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/1line1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert_eq!(stream.read_line().unwrap().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/1line2");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert_eq!(stream.read_line().unwrap().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/2lines1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert_eq!(stream.read_line().unwrap().as_slice(), "hello world!");
    assert_eq!(stream.read_line().unwrap().as_slice(), "bye bye world!");
    assert!(!stream.read_line().is_ok());

    expected = String::from_char(62, 'x');
    path = Path::new("tests/stream/xlines1");
    file = File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert_eq!(stream.read_line().unwrap(), expected);
    assert_eq!(stream.read_line().unwrap(), expected);
    assert_eq!(stream.read_line().unwrap(), expected);
    assert_eq!(stream.read_line().unwrap(), expected);
    assert_eq!(stream.read_line().unwrap(), expected);
    assert_eq!(stream.read_line().unwrap(), expected);
    assert_eq!(stream.read_line().unwrap(), expected);
    assert_eq!(stream.read_line().unwrap(), expected);
    assert_eq!(stream.read_line().unwrap(), expected);
    assert!(!stream.read_line().is_ok());
}
