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

use super::SmtpServerConfig;
use super::SmtpServerEventHandler;
use super::super::common::stream::{SmtpStream};
use super::super::common::utils;
use super::super::common::mailbox::Mailbox;
use super::super::common::transaction::{SmtpTransactionState, Init, Helo, Mail, Rcpt, Data};

// TODO: make SMTP handlers registerable by the library user so we can easily
// add commands and make the server extendable.
pub struct SmtpHandler<S: Writer+Reader, E: SmtpServerEventHandler> {
    pub command_start: String,
    pub allowed_states: Vec<SmtpTransactionState>,
    pub callback: fn(&mut SmtpStream<S>, &mut SmtpTransactionState, &SmtpServerConfig, &mut E, &str) -> Result<String, Option<String>>
}

impl<S: Writer+Reader, E: SmtpServerEventHandler> SmtpHandler<S, E> {
    fn new(command_start: &str, allowed_states: &[SmtpTransactionState], callback: fn(&mut SmtpStream<S>, &mut SmtpTransactionState, &SmtpServerConfig, &mut E, &str) -> Result<String, Option<String>>) -> SmtpHandler<S, E> {
        SmtpHandler {
            command_start: command_start.into_string(),
            allowed_states: allowed_states.to_vec(),
            callback: callback
        }
    }
}

pub fn get_handlers<S: Writer+Reader, E: SmtpServerEventHandler>() -> Vec<SmtpHandler<S, E>> {
    let all = [Init, Helo, Mail, Rcpt, Data];
    let handlers = vec!(
        SmtpHandler::new("HELO ", [Init], handle_command_helo),
        SmtpHandler::new("EHLO ", [Init], handle_command_helo),
        SmtpHandler::new("MAIL FROM:", [Helo], handle_command_mail),
        SmtpHandler::new("RCPT TO:", [Mail, Rcpt], handle_command_rcpt),
        SmtpHandler::new("DATA", [Rcpt], handle_command_data),
        SmtpHandler::new("RSET", all, handle_command_rset),
        SmtpHandler::new("VRFY ", all, handle_command_vrfy),
        SmtpHandler::new("EXPN ", all, handle_command_expn),
        SmtpHandler::new("HELP", all, handle_command_help),
        SmtpHandler::new("NOOP", all, handle_command_noop),
        SmtpHandler::new("QUIT", all, handle_command_quit)
    );
    handlers
}

#[allow(unused_variable)]
fn handle_command_helo<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    if line.len() == 0 {
        Ok("501 Domain name not provided".into_string())
    } else if utils::get_domain_len(line) != line.len() {
        Ok("501 Domain name is invalid".into_string())
    } else {
        match event_handler.handle_domain(line) {
            Ok(_) => {
                *state = Helo;
                Ok("250 OK".into_string())
            },
            Err(_) => {
                Ok("550 Domain not taken".into_string())
            }
        }
    }
}

#[test]
fn test_command_helo() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_mail<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    if line.len() < 2 || line.char_at(0) != '<' || line.char_at(line.len() - 1) != '>' {
        Ok("501 Email address invalid, must start with < and end with >".into_string())
    } else if line == "<>" {
        match event_handler.handle_sender_address(None) {
            Ok(_) => {
                *state = Mail;
                Ok("250 OK".into_string())
            },
            Err(_) => {
                Ok("550 Mailnot available".into_string())
            }
        }
    } else {
        let mailbox_res = Mailbox::parse(line.slice(1, line.len() - 1));
        match mailbox_res {
            Err(err) => {
                Ok(format!("553 Email address invalid: {}", err))
            },
            Ok(mailbox) => {
                match event_handler.handle_sender_address(Some(&mailbox)) {
                    Ok(_) => {
                        *state = Mail;
                        Ok("250 OK".into_string())
                    },
                    Err(_) => {
                        Ok("550 Mailnot taken".into_string())
                    }
                }
            }
        }
    }
}

#[test]
fn test_command_mail() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_rcpt<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    // TODO: check maximum number of recipients? Maybe after the event handler
    // sends back `Ok(())`?
    if false {
        Ok("452 Too many recipients".into_string())
    } else if line.char_at(0) != '<' || line.char_at(line.len() - 1) != '>' {
        Ok("501 Email address invalid, must start with < and end with >".into_string())
    } else {
        let mailbox_res = Mailbox::parse(line.slice(1, line.len() - 1));
        match mailbox_res {
            Err(err) => {
                Ok(format!("553 Email address invalid: {}", err))
            },
            Ok(mailbox) => {
                match event_handler.handle_receiver_address(&mailbox) {
                    Ok(_) => {
                        *state = Rcpt;
                        Ok("250 OK".into_string())
                    },
                    Err(_) => {
                        Ok("550 Mailnot available".into_string())
                    }
                }
            }
        }
    }
}

#[test]
fn test_command_rcpt() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_data<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    if line.len() != 0 {
        Ok("501 No arguments allowed".into_string())
    } else {
        stream.write_line("354 Start mail input; end with <CRLF>.<CRLF>").unwrap();

        // Inform our event handler that mail data is about to be received.
        event_handler.handle_body_start().unwrap();
        
        let mut size = 0;
        loop {
            let read_line = stream.read_line();
            let ok = read_line.is_ok();

            if ok {
                let read_line = read_line.unwrap();

                // Here, we check that we have already got some data, which
                // means that we have read a line, which means we have just
                // seen `<CRLF>`. And then, we check if the current line
                // which we know to end with `<CRLF>` as well contains a
                // single dot.
                // All in all, this means we check for `<CRLF>.<CRLF>`.
                if size != 0 && read_line == &['.' as u8] {
                    break;
                }
                // TODO: support transparency. Here or in the reader ?

                event_handler.handle_body_part(read_line).unwrap();

                size += read_line.len();

                if size > config.max_message_size {
                    // TODO: add an error handler in the event handler?
                    return Ok(format!(
                        "552 Too much mail data, max {} bytes",
                        config.max_message_size
                    ));
                }
            } else {
                return Err(None);
            }
        }

        // Inform our event handler that all data has been received.
        event_handler.handle_body_end().unwrap();

        // We're all good !
        state.reset();
        Ok("250 OK".into_string())
    }
}

#[test]
fn test_command_data() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_rset<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    if line.len() != 0 {
        Ok("501 No arguments allowed".into_string())
    } else {
        state.reset();
        Ok("250 OK".into_string())
    }
}

#[test]
fn test_command_rset() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_vrfy<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    Ok("252 Cannot VRFY user".into_string())
}

#[test]
fn test_command_vrfy() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_expn<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    Ok("252 Cannot EXPN mailing list".into_string())
}

#[test]
fn test_command_expn() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_help<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    if line.len() == 0 || line.char_at(0) == ' ' {
        Ok("502 Command not implemented".into_string())
    } else {
        Ok("500 Command unrecognized".into_string())
    }
}

#[test]
fn test_command_help() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_noop<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    if line.len() == 0 || line.char_at(0) == ' ' {
        Ok("250 OK".into_string())
    } else {
        Ok("500 Command unrecognized".into_string())
    }
}

#[test]
fn test_command_noop() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_quit<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       state: &mut SmtpTransactionState,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<String, Option<String>> {
    Err(Some(format!("221 {}", config.domain)))
}

#[test]
fn test_command_quit() {
    // fail!();
}
