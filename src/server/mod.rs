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

//! The `server` module contains things needed to build an SMTP server, but useless for
//! an SMTP client.

use std::io::net::tcp::{TcpListener, TcpAcceptor, TcpStream};
use std::io::{Listener, Acceptor, IoError, Reader, Writer};
use super::common::stream::{SmtpStream, LineTooLong};
use std::sync::Arc;
use std::ascii::OwnedAsciiExt;
use super::common::transaction::SmtpTransaction;
use super::common::mailbox::Mailbox;

mod handler;

/// Hooks into different places of the SMTP server to allow its customization.
pub trait SmtpServerEventHandler {
    /// Called after getting a recipient mailbox.
    ///
    /// If `Ok(())` is returned, a 250 response is sent. If `Err(())` is returned, a 550 response
    /// is sent and the recipient is discarded.
    #[allow(unused_variable)]
    fn handle_rcpt(&mut self, transaction: &SmtpTransaction, mailbox: &Mailbox) -> Result<(), ()> {
        Ok(())
    }

    /// Called after getting the entire message for this transaction.
    ///
    /// At this point, the client has not yet been notified of success or failure. If `Ok(())` is
    /// returned, a success message will be sent, else a `554 Transaction failed` message will be
    /// be sent and the transaction will be discarded.
    #[allow(unused_variable)]
    fn handle_transaction(&mut self, transaction: &SmtpTransaction) -> Result<(), ()> {
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
    pub domain: &'static str,
    /// The maximum message size, including headers and ending sequence.
    pub max_message_size: uint,
    //pub timeout: uint, // at least 5 minutes
    //pub max_clients: uint, // maximum clients to handle at any given time
    //pub max_pending_clients: uint, // maximum clients to put on hold while handling other clients
}

/// Represents an SMTP server which handles client transactions with any kind of stream.
///
/// This is useful for testing purposes as we can test the server from a plain text file. It
/// should not be used for other purposes directly. Use `SmtpServer` instead.
pub struct SmtpServer<S: 'static+Writer+Reader, A: Acceptor<S>, E: 'static+SmtpServerEventHandler> {
    // Underlying acceptor that allows accepting client connections to handle them.
    acceptor: A,
    config: Arc<SmtpServerConfig>,
    event_handler: E,
    handlers: Arc<Vec<handler::SmtpHandler<S, E>>>
}

/// Represents an error during creation of an SMTP server.
#[deriving(Show)]
pub enum SmtpServerError {
    /// The system call `bind` failed.
    BindFailed(IoError),
    /// The system call `listen` failed.
    ListenFailed(IoError),
}

#[test]
fn test_smtp_server_error() {
    // fail!();
}

impl<E: SmtpServerEventHandler+Clone+Send> SmtpServer<TcpStream, TcpAcceptor, E> {
    /// Creates a new SMTP server that listens on `0.0.0.0:2525`.
    pub fn new(config: SmtpServerConfig, event_handler: E) -> Result<SmtpServer<TcpStream, TcpAcceptor, E>, SmtpServerError> {
        match TcpListener::bind(config.ip, config.port) {
            Ok(listener) => {
                if config.debug {
                    println!("rsmtp: info: binding on ip {}", config.ip);
                }
                match listener.listen() {
                    Ok(acceptor) => {
                        if config.debug {
                            println!("rsmtp: info: listening on port {}", config.port);
                        }
                        Ok(SmtpServer::new_from_acceptor(acceptor, config, event_handler))
                    },
                    Err(err) => Err(ListenFailed(err))
                }
            },
            Err(err) => Err(BindFailed(err))
        }
    }
}

impl<S: Writer+Reader+Send, A: Acceptor<S>, E: SmtpServerEventHandler+Clone+Send> SmtpServer<S, A, E> {
    /// Creates a new SMTP server from an `Acceptor` implementor. Useful for testing.
    pub fn new_from_acceptor(acceptor: A, config: SmtpServerConfig, event_handler: E) -> SmtpServer<S, A, E> {
        SmtpServer {
            acceptor: acceptor,
            config: Arc::new(config),
            event_handler: event_handler,
            handlers: Arc::new(handler::get_handlers::<S, E>())
        }
    }

    /// Run the SMTP server.
    pub fn run(&mut self) {
        for mut stream_res in self.acceptor.incoming() {
            match stream_res {
                Ok(stream) => {
                    let handlers = self.handlers.clone();
                    let config = self.config.clone();
                    let mut event_handler = self.event_handler.clone();
                    spawn(proc() {
                        let mut stream = SmtpStream::new(stream, config.max_message_size);
                        // WAIT FOR: https://github.com/rust-lang/rust/issues/15802
                        //stream.stream.set_deadline(local_config.timeout);
                        let mut transaction = SmtpTransaction::new();

                        // Send the opening welcome message.
                        stream.write_line(format!("220 {}", config.domain).as_slice()).unwrap();

                        // Debug arrival of this client.
                        if config.debug {
                            println!("rsmtp: omsg: 220 {}", config.domain);
                        }

                        // Forever, looooop over command lines and handle them.
                        'main_loop: loop {
                            match stream.read_line() {
                                Ok(bytes) => {
                                    let line = String::from_utf8_lossy(bytes.as_slice()).into_string();

                                    if config.debug {
                                        println!("rsmtp: imsg: '{}'", line);
                                    }

                                    // Check if the line is a valid command. If so, do what needs to be done.
                                    for h in handlers.deref().iter() {
                                        // Don't check lines shorter than required. This also avoids getting an
                                        // out of bounds error below.
                                        if line.len() < h.command_start.len() {
                                            continue;
                                        }
                                        let line_start = line.as_slice().slice_to(h.command_start.len())
                                            .into_string().into_ascii_upper();
                                        // Check that the begining of the command matches an existing SMTP
                                        // command. This could be something like "HELO " or "RCPT TO:".
                                        if line_start.as_slice().starts_with(h.command_start.as_slice()) {
                                            if h.allowed_states.contains(&transaction.state) {
                                                let rest = line.as_slice().slice_from(h.command_start.len());
                                                // We're good to go!
                                                (h.callback)(
                                                    &mut stream,
                                                    &mut transaction,
                                                    config.deref(),
                                                    &mut event_handler,
                                                    rest
                                                ).unwrap();
                                                continue 'main_loop;
                                            } else {
                                                // Bad sequence of commands.
                                                stream.write_line("503 Bad sequence of commands").unwrap();
                                                // Debug to console.
                                                if config.debug {
                                                    println!("rsmtp: omsg: 503 Bad sequence of commands");
                                                }
                                                continue 'main_loop;
                                            }
                                        }
                                    }
                                },
                                Err(err) => {
                                    // If the line was too long, notify the client.
                                    if err == LineTooLong {
                                        stream.write_line("500 Command line too long, max is 512 bytes").unwrap();
                                        // Debug to console.
                                        if config.debug {
                                            println!("rsmtp: omsg: 500 Command line too long, max is 512 bytes");
                                        }
                                        continue 'main_loop;
                                    } else {
                                        // If we get here, the error is unexpected. What to do with it?
                                        fail!(err);
                                    }
                                }
                            }
                            // No valid command was given.
                            stream.write_line("500 Command unrecognized").unwrap();
                            // Debug to console.
                            if config.debug {
                                println!("rsmtp: omsg: 500 Command unrecognized");
                            }
                        }
                    });
                },
                // Ignore accept error. Is this right? If you think not, please open an issue on Github.
                _ => {}
            }
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
