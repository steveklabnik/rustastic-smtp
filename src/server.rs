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

use std::io::net::tcp::{TcpListener, TcpAcceptor, TcpStream};
use std::io::{Listener, Acceptor, Reader, Writer};
use super::stream::{SmtpStream};
use super::mailbox::{Mailbox};
use super::{utils};
use std::sync::{Arc};
use std::ascii::{OwnedAsciiExt};

/// Hooks into different places of the SMTP server to allow its customization.
pub trait SmtpServerEventHandler {
    /// Called after getting the sender mailbox. If `Err(())` is returned, a 550 response is sent.
    #[allow(unused_variable)]
    fn handle_mail(&mut self, mailbox: &Mailbox) -> Result<(), ()> {
        Ok(())
    }

    /// Called after getting a recipient mailbox. If `Err(())` is returned, a 550 response is sent.
    #[allow(unused_variable)]
    fn handle_rcpt(&mut self, mailbox: &Mailbox) -> Result<(), ()> {
        Ok(())
    }

    #[allow(unused_variable)]
    fn handle_transaction(&mut self, transaction: &SmtpTransaction) -> Result<(), ()> {
        Ok(())
    }

    #[allow(unused_variable)]
    fn handle_error(&mut self, err: &SmtpServerError) -> Result<(), ()> {
        Ok(())
    }
}

/// Represents the configuration of an SMTP server.
pub struct SmtpServerConfig {
    /// Maximum number of recipients per SMTP transaction.
    pub max_recipients: uint,
    /// Port on which to listen for incoming messages.
    pub port: u16,
    /// If `true`, debug messages will be printed to the console during transactions.
    pub debug: bool,
    /// The IP on which to `bind (2)` the `TcpListener`.
    pub ip: &'static str,
    /// The domain name used to identify the SMTP server.
    pub domain: &'static str
    //pub timeout: uint, // at least 5 minutes
    //pub max_clients: uint, // maximum clients to handle at any given time
    //pub max_pending_clients: uint, // maximum clients to put on hold while handling other clients
    //pub max_message_size: uint, // at least 2 ^ 16
}

/// Represents an SMTP server which handles client transactions with any kind of stream.
///
/// This is useful for testing purposes as we can test the server from a plain text file. It
/// should not be used for other purposes directly. Use `SmtpServer` instead.
pub struct SmtpServer<S: 'static+Writer+Reader, A: Acceptor<S>, E: 'static+SmtpServerEventHandler> {
    // Underlying acceptor that allows accepting client connections to handle them.
    acceptor: A,
    config: Arc<SmtpServerConfig>,
    event_handler: E
}

/// Represents an error during creation of an SMTP server.
#[deriving(Show)]
pub enum SmtpServerError {
    /// The system call `bind` failed.
    BindFailed,
    /// The system call `listen` failed.
    ListenFailed
}

#[test]
fn test_smtp_server_error() {
    // fail!();
}

/// Represents the current state of an SMTP transaction.
///
/// This is useful for checking if an incoming SMTP command is allowed at any given moment
/// during an SMTP transaction.
#[deriving(PartialEq, Eq, Clone)]
pub enum SmtpTransactionState {
    Init,
    Helo,
    Mail,
    Rcpt,
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
    /// The email address of the sender.
    pub from: Mailbox,
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
            // Put a default email address. This will never be accessed unless replaced. Also,
            // since "r@r" is valid, we can `unwrap()` safely.
            from: Mailbox::parse("r@r").unwrap(),
            data: Vec::new(),
            state: Init
        }
    }

    /// Resets the `to`, `from` and `data` fields, as well as the `state` of the transaction.
    ///
    /// This is used when a transaction ends and when `RSET` is sent by the client.
    pub fn reset(&mut self) {
        self.to = Vec::new();
        self.from = Mailbox::parse("r@r").unwrap();
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

impl<E: SmtpServerEventHandler+Clone+Send> SmtpServer<TcpStream, TcpAcceptor, E> {
    /// Creates a new SMTP server that listens on `0.0.0.0:2525`.
    pub fn new(config: SmtpServerConfig, event_handler: E) -> Result<SmtpServer<TcpStream, TcpAcceptor, E>, SmtpServerError> {
        let listener = TcpListener::bind(config.ip, config.port).unwrap();
        if config.debug {
            println!("rsmtp: info: binding on ip {}", config.ip);
        }
        let acceptor = listener.listen().unwrap();
        if config.debug {
            println!("rsmtp: info: listening on port {}", config.port);
        }
        SmtpServer::new_from_acceptor(acceptor, config, event_handler)
    }
}

impl<S: Writer+Reader+Send, A: Acceptor<S>, E: SmtpServerEventHandler+Clone+Send> SmtpServer<S, A, E> {
    /// Creates a new SMTP server from an `Acceptor` implementor. Useful for testing.
    pub fn new_from_acceptor(acceptor: A, config: SmtpServerConfig, event_handler: E) -> Result<SmtpServer<S, A, E>, SmtpServerError> {
        Ok(SmtpServer {
            acceptor: acceptor,
            config: Arc::new(config),
            event_handler: event_handler
        })
    }

    fn handlers<S: Writer+Reader, E: SmtpServerEventHandler>(&self) -> Vec<(
        // The prefix in the command sent by the client.
        String,
        // The list of allowed states for this command.
        Vec<SmtpTransactionState>,
        // The handler function to call for this command.
        fn(&mut SmtpStream<S>, &mut SmtpTransaction,
           &SmtpServerConfig, &mut E, &str) -> Result<(), ()>
    )> {
        let all = &[Init, Helo, Mail, Rcpt, Data];
        let handlers = vec!(
            ("HELO ".into_string(),[Init].into_vec(), handle_command_helo),
            ("EHLO ".into_string(), [Init].into_vec(), handle_command_helo),
            ("MAIL FROM:".into_string(), [Helo].into_vec(), handle_command_mail),
            ("RCPT TO:".into_string(), [Mail, Rcpt].into_vec(), handle_command_rcpt),
            ("DATA".into_string(), [Rcpt].into_vec(), handle_command_data),
            ("RSET".into_string(), all.into_vec(), handle_command_rset),
            ("VRFY ".into_string(), all.into_vec(), handle_command_vrfy),
            ("EXPN ".into_string(), all.into_vec(), handle_command_expn),
            ("HELP".into_string(), all.into_vec(), handle_command_help),
            ("NOOP".into_string(), all.into_vec(), handle_command_noop),
            ("QUIT".into_string(), all.into_vec(), handle_command_quit)
        );
        handlers
    }

    /// Run the SMTP server.
    pub fn run(&mut self) {
        // Since cea
        let handlers = Arc::new(self.handlers());
        for mut stream_res in self.acceptor.incoming() {
            let local_handlers = handlers.clone();
            let local_config = self.config.clone();
            let mut local_event_handler = self.event_handler.clone();
            spawn(proc() {
                // TODO: is there a better way to handle an error here?
                let mut stream = SmtpStream::new(stream_res.unwrap());
                // WAIT FOR: https://github.com/rust-lang/rust/issues/15802
                //stream.stream.set_deadline(local_config.timeout);
                let mut transaction = SmtpTransaction::new();

                // Send the opening welcome message.
                stream.write_line(format!("220 {}", local_config.domain).as_slice()).unwrap();

                // Debug arrival of this client.
                if local_config.debug {
                    println!("rsmtp: omsg: 220 {}", local_config.domain);
                }

                // Forever, looooop over command lines and handle them.
                'main_loop: loop {
                    // Find the right handler.
                    // TODO: check the return value and return appropriate error message,
                    // ie "500 Command line too long".
                    let line = String::from_utf8_lossy(stream.read_line().unwrap().as_slice()).into_string();

                    if local_config.debug {
                        println!("rsmtp: imsg: '{}'", line);
                    }

                    // Check if the line is a valid command. If so, do what needs to be done.
                    for h in local_handlers.deref().iter() {
                        // Don't check lines shorter than required. This also avoids getting an
                        // out of bounds error below.
                        if line.len() < h.ref0().len() {
                            continue;
                        }
                        let line_start = line.as_slice().slice_to(h.ref0().len())
                            .into_string().into_ascii_upper();
                        // Check that the begining of the command matches an existing SMTP
                        // command. This could be something like "HELO " or "RCPT TO:".
                        if line_start.as_slice().starts_with(h.ref0().as_slice()) {
                            if h.ref1().contains(&transaction.state) {
                                let rest = line.as_slice().slice_from((*h.ref0()).len());
                                // We're good to go!
                                (*h.ref2())(
                                    &mut stream,
                                    &mut transaction,
                                    local_config.deref(),
                                    &mut local_event_handler,
                                    rest
                                ).unwrap(); // TODO: avoid unwrap here.
                                continue 'main_loop;
                            } else {
                                // Bad sequence of commands.
                                stream.write_line("503 Bad sequence of commands").unwrap();
                                // Debug to console.
                                if local_config.debug {
                                    println!("rsmtp: omsg: 503 Bad sequence of commands");
                                }
                                continue 'main_loop;
                            }
                        }
                    }
                    // No valid command was given.
                    stream.write_line("500 Command unrecognized").unwrap();

                    if local_config.debug {
                        println!("rsmtp: omsg: 500 Command unrecognized");
                    }
                }
            });
        }
    }
}

#[test]
fn test_smtp_server_new() {
    // fail!();
}

#[test]
fn test_smtp_server_new_from_acceptor() {
    // fail!();
}

#[test]
fn test_smtp_server_handlers() {
    // fail!();
}

#[test]
fn test_smtp_server_run() {
    // fail!();
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
        transaction.domain = line.into_string();
        transaction.state = Helo;
        stream.write_line("250 OK").unwrap();
        if config.debug {
            println!("rsmtp: omsg: 250 OK");
        }
        Ok(())
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
    if line.char_at(0) != '<' || line.char_at(line.len() - 1) != '>' {
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
                match event_handler.handle_mail(&mailbox) {
                    Ok(_) => {
                        transaction.from = mailbox;
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
                match event_handler.handle_rcpt(&mailbox) {
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
        transaction.data = stream.read_data().unwrap();
        transaction.state = Data;
        // Send an immutable reference of the transaction.
        match event_handler.handle_transaction(&*transaction) {
            Ok(_) => {
                transaction.reset();
                stream.write_line("250 OK").unwrap();
                if config.debug {
                    println!("rsmtp: omsg: 250 OK");
                }
            },
            Err(_) => {
                stream.write_line("554 Transaction failed").unwrap();
                if config.debug {
                    println!("rsmtp: omsg: 554 Transaction failed");
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
