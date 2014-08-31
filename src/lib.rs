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
//! use rsmtp::server::{SmtpServer, SmtpServerConfig};
//!
//! fn main() {
//!     let config = SmtpServerConfig {
//!         ip: "127.0.0.1",
//!         domain: "rustastic.org",
//!         port: 2525,
//!         max_recipients: 100,
//!         debug: true
//!     };
//!     let mut server = SmtpServer::new(config).unwrap();
//!     server.run();
//! }
//! ```

extern crate libc;

pub mod stream;
pub mod mailbox;
pub mod server;

// This is private at the moment because the API is far from stable.
mod utils;
