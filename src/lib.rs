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
//! use rsmtp::server::{SmtpServer, SmtpServerConfig, SmtpServerEventHandler, SmtpTransaction};
//!
//! #[deriving(Clone)]
//! struct Handler;
//!
//! impl Handler {
//!     fn save(&mut self, data: &SmtpTransaction) -> Result<(), ()> {
//!         // save to a database, send to an API, whatever you want :-)
//!         Ok(())
//!     }
//! }
//!
//! impl SmtpServerEventHandler for Handler {
//!     fn handle_transaction(&mut self, transaction: &SmtpTransaction) -> Result<(), ()> {
//!         self.save(transaction)
//!     }
//! }
//!
//! fn main() {
//!     let config = SmtpServerConfig {
//!         ip: "127.0.0.1",
//!         domain: "rustastic.org",
//!         port: 2525,
//!         max_recipients: 100,
//!         debug: true
//!     };
//!     let mut server = SmtpServer::new(config, Handler).unwrap();
//!     server.run();
//! }
//! ```

extern crate libc;

pub mod stream;
pub mod mailbox;
pub mod server;

// This is private at the moment because the API is far from stable.
mod utils;
