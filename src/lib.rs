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
//! use rsmtp::server::{
//!     SmtpServer,
//!     SmtpServerConfig,
//!     SmtpServerEventHandler,
//!     SmtpTransaction
//! };
//! use rsmtp::mailbox::{Mailbox};
//!
//! #[deriving(Clone)]
//! struct Handler;
//!
//! impl SmtpServerEventHandler for Handler {
//!     fn handle_rcpt(&mut self, transaction: &SmtpTransaction, mailbox: &Mailbox) -> Result<(), ()> {
//!         println!("Check in a database if this recipient is valid and more if you want.");
//!         Ok(())
//!     }
//!
//!     fn handle_transaction(&mut self, transaction: &SmtpTransaction) -> Result<(), ()> {
//!         println!("Save to a database, send to an API, whatever you want :-)");
//!         Ok(())
//!     }
//! }
//!
//! fn main() {
//!     let config = SmtpServerConfig {
//!         ip: "0.0.0.0",
//!         domain: "rustastic.org",
//!         port: 25,
//!         max_recipients: 100,
//!         max_message_size: 65536,
//!         debug: true
//!     };
//!     let mut server = SmtpServer::new(config, Handler).unwrap();
//!     server.run();
//! }
//! ```

#![feature(macro_rules)]

extern crate libc;

pub mod stream;
pub mod mailbox;
pub mod server;

// This is private at the moment because the API is far from stable.
mod utils;
