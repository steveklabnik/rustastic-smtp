use std::io::{Reader, IoResult, IoError};
use std::string::{String};
use libc::{EOF};

/// The maximum line size as specified by RFC 5321.
static MAX_LINE_SIZE: uint = 512;

/// A reader specially made for reading SMTP commands.
///
/// It reads lines of input delimited by the <CRLF> sequence and with a maximum
/// size of 512 bytes, including the command word and the <CRLF> sequence. If
/// the input is not UTF8, non-UTF8 characters are replaced with `U+FFFD
/// REPLACEMENT CHARACTER` but no error is returned.
///
/// Returns `EndOfFile` if no line is found within 512 bytes of input.
pub struct SmtpReader<R> {
    reader: R,
    vec: Vec<u8>
}

impl<R: Reader> SmtpReader<R> {
    /// Create a new `SmtpReader` from another reader.
    pub fn new(inner: R) -> SmtpReader<R> {
        SmtpReader {
            reader: inner,
            vec: Vec::with_capacity(MAX_LINE_SIZE)
        }
    }

    /// Read one line of input.
    pub fn read_line(&mut self) -> IoResult<String> {
        // First, we check if we have a line buffered already. If so, we return
        // it straightaway. Else, we read more input and then try to find a
        // line.
        match self.read_buffered_line() {
            Ok(res) => Ok(res),
            Err(_) => match self.buffer_line() {
                Err(e) => Err(e),
                Ok(_) => {
                    self.read_buffered_line()
                }
            }
        }
    }

    /// Get more input from the underlying reader.
    fn buffer_line(&mut self) -> IoResult<uint> {
        self.reader.push(
            MAX_LINE_SIZE - self.vec.len(),
            &mut self.vec
        )
    }

    /// Read a line from the already buffered input.
    fn read_buffered_line(&mut self) -> IoResult<String> {
        // Try to find a CRLF sequence. If none is found, we'll return
        // an error. Else, we'll return the String up to that sequence
        // and rearrange the reader vector so no data is lost.
        let or = self.vec.as_slice().position_elem(&13u8);
        let on = self.vec.as_slice().position_elem(&10u8);
        match (or, on) {
            (Some(posr), Some(posn)) => if posn == posr + 1 {
                let s = String::from_utf8_lossy(
                    self.vec.slice_to(posr)
                ).into_string();
                self.vec = self.vec.slice_from(posn + 1).into_vec();
                Ok(s)
            } else {
                Err(IoError::from_errno(EOF as uint, true))
            },
            _ => Err(IoError::from_errno(EOF as uint, true))
        }
    }
}

#[test]
fn test_reader() {}
