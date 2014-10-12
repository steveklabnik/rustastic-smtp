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

use std::io::{InvalidInput};
use super::SmtpServerConfig;
use super::SmtpServerEventHandler;
use super::super::common::stream::{SmtpStream};
use super::super::common::utils;
use super::super::common::mailbox::Mailbox;
use super::super::common::transaction::{SmtpTransaction, SmtpTransactionState, Init, Helo, Mail, Rcpt, Data};

// TODO: make SMTP handlers registerable by the library user so we can easily
// add commands and make the server extendable.
pub struct SmtpHandler<S: Writer+Reader, E: SmtpServerEventHandler> {
    pub command_start: String,
    pub allowed_states: Vec<SmtpTransactionState>,
    pub callback: fn(&mut SmtpStream<S>, &mut SmtpTransaction, &SmtpServerConfig, &mut E, &str) -> Result<(), ()>
}

impl<S: Writer+Reader, E: SmtpServerEventHandler> SmtpHandler<S, E> {
    fn new(command_start: &str, allowed_states: &[SmtpTransactionState], callback: fn(&mut SmtpStream<S>, &mut SmtpTransaction, &SmtpServerConfig, &mut E, &str) -> Result<(), ()>) -> SmtpHandler<S, E> {
        SmtpHandler {
            command_start: command_start.into_string(),
            allowed_states: allowed_states.into_vec(),
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
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    if line.len() == 0 {
        stream.write_line("501 Domain name not provided").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 501 Domain name is invalid");
        }
        Ok(())
    } else if utils::get_domain_len(line) != line.len() {
        stream.write_line("501 Domain name is invalid").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 501 Domain name is invalid");
        }
        Ok(())
    } else {
        match event_handler.handle_domain(line) {
            Ok(_) => {
                transaction.domain = line.into_string();
                transaction.state = Helo;
                stream.write_line("250 OK").unwrap();
                if config.debug {
                    println!("rsmtp: omsg: 250 OK");
                }
                Ok(())
            },
            Err(_) => {
                stream.write_line("550 Domain not taken").unwrap();
                if config.debug {
                    println!("rsmtp: omsg: 550 Domain not taken");
                }
                Err(())
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
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    if line.len() < 2 || line.char_at(0) != '<' || line.char_at(line.len() - 1) != '>' {
        stream.write_line("501 Email address invalid, must start with < and end with >").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 501 Email address invalid, must start with < and end with >");
        }
        Ok(())
    } else if line == "<>" {
        match event_handler.handle_sender_address(None) {
            Ok(_) => {
                transaction.from = None;
                transaction.state = Mail;
                stream.write_line("250 OK").unwrap();
                if config.debug {
                    println!("rsmtp: omsg: 250 OK");
                }
                Ok(())
            },
            Err(_) => {
                stream.write_line("550 Mailbox not available").unwrap();
                if config.debug {
                    println!("rsmtp: omsg: 550 Mailbox not available");
                }
                Err(())
            }
        }
    } else {
        let mailbox_res = Mailbox::parse(line.slice(1, line.len() - 1));
        match mailbox_res {
            Err(err) => {
                stream.write_line(format!("553 Email address invalid: {}", err).as_slice())
                    .unwrap();
                if config.debug {
                    println!("rsmtp: omsg: 553 Email address invalid: {}", err);
                }
            },
            Ok(mailbox) => {
                match event_handler.handle_sender_address(Some(&mailbox)) {
                    Ok(_) => {
                        transaction.from = Some(mailbox);
                        transaction.state = Mail;
                        stream.write_line("250 OK").unwrap();
                        if config.debug {
                            println!("rsmtp: omsg: 250 OK");
                        }                        
                    },
                    Err(_) => {
                        stream.write_line("550 Mailbox not taken").unwrap();
                        if config.debug {
                            println!("rsmtp: omsg: 550 Mailbox not taken");
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[test]
fn test_command_mail() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_rcpt<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    if transaction.to.len() >= config.max_recipients {
        stream.write_line("452 Too many recipients").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 452 Too many recipients");
        }
        Ok(())
    } else if line.char_at(0) != '<' || line.char_at(line.len() - 1) != '>' {
        stream.write_line("501 Email address invalid, must start with < and end with >").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 501 Email address invalid, must start with < and end with >");
        }
        Ok(())
    } else {
        let mailbox_res = Mailbox::parse(line.slice(1, line.len() - 1));
        match mailbox_res {
            Err(err) => {
                stream.write_line(format!("553 Email address invalid: {}", err).as_slice())
                    .unwrap();
                if config.debug {
                    println!("rsmtp: omsg: 553 Email address invalid: {}", err);
                }
            },
            Ok(mailbox) => {
                match event_handler.handle_receiver_address(&mailbox) {
                    Ok(_) => {
                        transaction.to.push(mailbox);
                        transaction.state = Rcpt;
                        stream.write_line("250 OK").unwrap();
                        if config.debug {
                            println!("rsmtp: omsg: 250 OK");
                        }
                    },
                    Err(_) => {
                        stream.write_line("550 Mailbox not available").unwrap();
                        if config.debug {
                            println!("rsmtp: omsg: 550 Mailbox not available");
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[test]
fn test_command_rcpt() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_data<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    if line.len() != 0 {
        stream.write_line("501 No arguments allowed").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 501 No arguments allowed");
        }
    } else {
        stream.write_line("354 Start mail input; end with <CRLF>.<CRLF>").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 354 Start mail input; end with <CRLF>.<CRLF>");
        }
        match stream.read_data() {
            Ok(data) => {
                transaction.data = data;
                transaction.state = Data;
                // // Send an immutable reference of the transaction.
                // match event_handler.handle_transaction(&*transaction) {
                //     Ok(_) => {
                        transaction.reset();
                        stream.write_line("250 OK").unwrap();
                        if config.debug {
                            println!("rsmtp: omsg: 250 OK");
                        }
                //     },
                //     Err(_) => {
                //         stream.write_line("554 Transaction failed").unwrap();
                //         if config.debug {
                //             println!("rsmtp: omsg: 554 Transaction failed");
                //         }
                //     }
                // }
            },
            Err(err) => {
                match err.kind {
                    InvalidInput => {
                        let msg = format!(
                            "552 Too much mail data, max {} bytes",
                            config.max_message_size
                        );
                        stream.write_line(msg.as_slice()).unwrap();
                        if config.debug {
                            println!("rsmtp: omsg: {}", msg);
                        }
                    },
                    _ => {
                        // Unexpected error, what do we do?
                        fail!()
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn test_command_data() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_rset<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    if line.len() != 0 {
        stream.write_line("501 No arguments allowed").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 501 No arguments allowed");
        }
    } else {
        transaction.reset();
        stream.write_line("250 OK").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 250 OK");
        }
    }
    Ok(())
}

#[test]
fn test_command_rset() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_vrfy<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    stream.write_line("252 Cannot VRFY user").unwrap();
    if config.debug {
        println!("rsmtp: omsg: 252 Cannot VRFY user");
    }
    Ok(())
}

#[test]
fn test_command_vrfy() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_expn<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    stream.write_line("252 Cannot EXPN mailing list").unwrap();
    if config.debug {
        println!("rsmtp: omsg: 252 Cannot EXPN mailing list");
    }
    Ok(())
}

#[test]
fn test_command_expn() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_help<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    if line.len() == 0 || line.char_at(0) == ' ' {
        stream.write_line("502 Command not implemented").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 502 Command not implemented");
        }
    } else {
        stream.write_line("500 Command unrecognized").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 500 Command unrecognized");
        }
    }
    Ok(())
}

#[test]
fn test_command_help() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_noop<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    if line.len() == 0 || line.char_at(0) == ' ' {
        stream.write_line("250 OK").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 250 OK");
        }
    } else {
        stream.write_line("500 Command unrecognized").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 500 Command unrecognized");
        }
    }
    Ok(())
}

#[test]
fn test_command_noop() {
    // fail!();
}

#[allow(unused_variable)]
fn handle_command_quit<S: Writer+Reader, E: SmtpServerEventHandler>(stream: &mut SmtpStream<S>,
                       transaction: &mut SmtpTransaction,
                       config: &SmtpServerConfig,
                       event_handler: &mut E,
                       line: &str) -> Result<(), ()> {
    stream.write_line(format!("221 {}", config.domain).as_slice()).unwrap();
    if config.debug {
        println!("rsmtp: omsg: 221 {}", config.domain);
    }
    Err(())
}

#[test]
fn test_command_quit() {
    // fail!();
}
