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
//! use rsmtp::server::{SmtpServer, SmtpServerEventHandler, SmtpServerConfig};
//! use rsmtp::common::mailbox::Mailbox;
//! use rsmtp::common::{
//!     MIN_ALLOWED_MESSAGE_SIZE,
//!     MIN_ALLOWED_LINE_SIZE,
//!     MIN_ALLOWED_RECIPIENTS
//! };
//! use std::io::net::ip::IpAddr;
//!
//! #[deriving(Clone)]
//! struct Handler;
//!
//! impl SmtpServerEventHandler for Handler {
//!     fn handle_connection(&mut self, client_ip: &IpAddr) -> Result<(), ()> {
//!         Ok(())
//!     }
//!     fn handle_sender_address(&mut self, mailbox: Option<&Mailbox>) -> Result<(), ()> {
//!         Ok(())
//!     }
//! }
//!
//! fn main() {
//!     let config = SmtpServerConfig {
//!         ip: "0.0.0.0",
//!         domain: "rustastic.org",
//!         port: 25,
//!         max_recipients: MIN_ALLOWED_RECIPIENTS,
//!         max_message_size: MIN_ALLOWED_MESSAGE_SIZE,
//!         max_line_size: MIN_ALLOWED_LINE_SIZE,
//!         debug: true
//!     };
//!     let mut server = SmtpServer::new(config, Handler).unwrap();
//!     server.run();
//! }
//! ```

#![deny(unnecessary_qualification)]
#![deny(non_uppercase_statics)]
#![deny(unnecessary_typecast)]
#![deny(missing_doc)]
#![deny(unused_result)]

pub mod client;
pub mod common;
pub mod server;
