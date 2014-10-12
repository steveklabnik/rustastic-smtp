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

impl SmtpTransactionState {
    /// Reset the state.
    pub fn reset(&mut self) {
        match *self {
            Init => {
                // Do nothing.
            },
            _ => {
                *self = Helo;
            }
        }
    }
}

#[test]
fn test_smtp_transaction_state() {
    // fail!();
}
