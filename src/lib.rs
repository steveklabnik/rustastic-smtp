//! Rustastic SMTP is meant to provide SMTP tools such as email address parsing
//! utilities as well as a configurable SMTP server and client.
//!
//! The goal is to eventually comply with the
//! [SMTP spec from RFC 5321](http://tools.ietf.org/html/rfc5321).
//!
//! # Example
//!
//!
//! extern crate rsmtp;
//!
//! use rsmtp::server;
//!
//! fn main() {
//!     let mut server = server::SmtpServer::new().unwrap();
//!     println!("Listening on port 2525...");
//!     server.run();
//! }
//! 

extern crate libc;

pub mod stream;
pub mod mailbox;
pub mod server;

// This is private at the moment because the API is far from stable.
mod utils;
