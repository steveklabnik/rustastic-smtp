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

//! Tools for managing the state of a connection between an SMTP client and an SMTP server.

use super::super::common::mailbox::Mailbox;

// TODO: make transaction states extendable, like:
// Core(Init) and Custom(XMySmtpState) / Custom("X-MY-SMTP-STATE")

/// Represents the current state of an SMTP transaction.
///
/// This is useful for checking if an incoming SMTP command is allowed at any given moment
/// during an SMTP transaction.
#[deriving(PartialEq, Eq, Clone)]
pub enum SmtpTransactionState {
    /// The initial state, when no commands have been sent by the client yet.
    Init,
    /// The client has sent `EHLO` or `HELO`.
    Helo,
    /// The client has sent `MAIL FROM`.
    Mail,
    /// The client has sent at least one `RCPT TO`.
    Rcpt,
    /// The client has sent `DATA.
    Data
}

#[test]
fn test_smtp_transaction_state() {
    // fail!();
}

/// Represents an SMTP transaction.
pub struct SmtpTransaction {
    /// Domain name passed via `HELO`/`EHLO`.
    pub domain: String,
    /// A vector of recipients' email addresses.
    pub to: Vec<Mailbox>,
    /// The email address of the sender or `None` if it was `<>`.
    pub from: Option<Mailbox>,
    /// The body of the email.
    pub data: Vec<u8>,
    /// The current state of the transaction.
    pub state: SmtpTransactionState
}

impl SmtpTransaction {
    /// Creates a new transaction.
    pub fn new() -> SmtpTransaction {
        SmtpTransaction {
            domain: String::new(),
            to: Vec::new(),
            from: None,
            data: Vec::new(),
            state: Init
        }
    }

    /// Resets the `to`, `from` and `data` fields, as well as the `state` of the transaction.
    ///
    /// This is used when a transaction ends and when `RSET` is sent by the client.
    pub fn reset(&mut self) {
        self.to = Vec::new();
        self.from = None;
        self.data = Vec::new();
        if self.state != Init {
            self.state = Helo;
        }
    }
}

#[test]
fn test_smtp_transaction_new() {
    // fail!();
}

#[test]
fn test_smtp_transaction_reset() {
    // fail!();
}
