use std::io::{Reader, Writer, IoErrorKind, InvalidInput};
use std::string::{String};

/// The maximum line size as specified by RFC 5321.
static MAX_LINE_SIZE: uint = 512;

/// A stream specially made for reading SMTP commands.
///
/// It reads lines of input delimited by the <CRLF> sequence and with a maximum
/// size of 512 bytes, including the command word and the <CRLF> sequence. If
/// the input is not UTF8, non-UTF8 characters are replaced with `U+FFFD
/// REPLACEMENT CHARACTER` but no error is returned.
///
/// Returns `EndOfFile` if no line is found within 512 bytes of input.
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
                Err(e) => return Err(e.kind)
            }
        }
    }

}

impl<W: Writer> SmtpStream<W> {
    /// Write a line ended with CRLF.
    pub fn write_line(&mut self, s: &str) -> Result<(), ()> {
        match self.stream.write_str(format!("{}\r\n", s).as_slice()) {
            Ok(_) => Ok(()),
            Err(_) => Err(())
        }
    }
}

#[test]
fn test_reader() {
    let mut path: Path;
    let mut file: super::std::io::fs::File;
    let mut stream: SmtpStream<super::std::io::fs::File>;
    let mut expected: String;

    path = Path::new("tests/stream/0line1");
    file = super::std::io::fs::File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line2");
    file = super::std::io::fs::File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/0line3");
    file = super::std::io::fs::File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/1line1");
    file = super::std::io::fs::File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert_eq!(stream.read_line().unwrap().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/1line2");
    file = super::std::io::fs::File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert_eq!(stream.read_line().unwrap().as_slice(), "hello world!");
    assert!(!stream.read_line().is_ok());

    path = Path::new("tests/stream/2lines1");
    file = super::std::io::fs::File::open(&path).unwrap();
    stream = SmtpStream::new(file);
    assert_eq!(stream.read_line().unwrap().as_slice(), "hello world!");
    assert_eq!(stream.read_line().unwrap().as_slice(), "bye bye world!");
    assert!(!stream.read_line().is_ok());

    expected = String::from_char(62, 'x');
    path = Path::new("tests/stream/xlines1");
    file = super::std::io::fs::File::open(&path).unwrap();
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
