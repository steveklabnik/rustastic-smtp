//! Rustastic SMTP is meant to provide SMTP tools such as email address parsing
//! utilities as well as a configurable SMTP server and client.
//!
//! The goal is to eventually comply with the
//! [SMTP spec from RFC 5321](http://tools.ietf.org/html/rfc5321).
//!
//! # Example
//!
//! ```no_run
//! extern crate rsmtp;
//!
//! use rsmtp::server::{SmtpServer};
//! use std::io::net::tcp::{TcpListener};
//! use std::io::{Listener};
//!
//! fn main() {
//!     let listener = TcpListener::bind("0.0.0.0", 2525).unwrap();
//!     let acceptor = listener.listen().unwrap();
//!     let mut server = SmtpServer::new(acceptor).unwrap();
//!     println!("Listening on port 2525...");
//!     server.run();
//! }
//! ```

extern crate libc;

pub mod stream;
pub mod mailbox;
pub mod server;

// This is private at the moment because the API is far from stable.
mod utils;
