use std::io::net::tcp::{TcpListener, TcpAcceptor, TcpStream};
use std::io::{Listener, Acceptor};
use super::stream::{SmtpStream};
use super::mailbox::{Mailbox};
use super::{utils};

static handlers: &'static [HandlerDescription] = &[
    ("HELO ", &[Init], handle_command_helo),
    ("EHLO", &[Init], handle_command_helo),
    ("MAIL FROM:", &[Helo], handle_command_mail),
    ("RCPT TO:", &[Mail, Rcpt], handle_command_rcpt),
    ("DATA", &[Rcpt], handle_command_data),
    ("RSET", &[Init, Helo, Mail, Rcpt, Data], handle_command_rset),
    ("VRFY ", &[Init, Helo, Mail, Rcpt, Data], handle_command_vrfy),
    ("EXPN ", &[Init, Helo, Mail, Rcpt, Data], handle_command_expn),
    ("HELP", &[Init, Helo, Mail, Rcpt, Data], handle_command_help),
    ("NOOP", &[Init, Helo, Mail, Rcpt, Data], handle_command_noop),
    ("QUIT", &[Init, Helo, Mail, Rcpt, Data], handle_command_quit)
];

type HandlerDescription = (
    // The prefix in the command sent by the client.
    &'static str,
    // The list of allowed states for this command.
    &'static [SmtpTransactionState],
    // The handler function to call for this command.
    HandlerFunction
);

type HandlerFunction = fn(&mut SmtpStream<TcpStream>,
                          &mut SmtpTransaction, &str) -> Result<(), ()>;

pub struct SmtpServer {
    acceptor: TcpAcceptor
}

#[deriving(Show)]
pub enum SmtpServerError {
    BindFailed,
    ListenFailed
}

#[deriving(PartialEq, Eq)]
pub enum SmtpTransactionState {
    Init,
    Helo,
    Mail,
    Rcpt,
    Data
}

pub struct SmtpTransaction {
    domain: Option<String>,
    to: Vec<Mailbox>,
    from: Option<Mailbox>,
    data: Option<String>,
    state: SmtpTransactionState
}

impl SmtpTransaction {
    pub fn new() -> SmtpTransaction {
        SmtpTransaction {
            domain: None,
            to: Vec::new(),
            from: None,
            data: None,
            state: Init
        }
    }

    pub fn reset(&mut self) {
        self.to.clear();
        self.from = None;
        self.data = None;
        if self.state != Init {
            self.state = Helo;
        }
    }
}

impl SmtpServer {
    pub fn new() -> Result<SmtpServer, SmtpServerError> {
        let listener_res = TcpListener::bind("0.0.0.0", 2525);
        if listener_res.is_err() {
            return Err(BindFailed)
        }
        let listener = listener_res.unwrap();

        let acceptor_res = listener.listen();
        if acceptor_res.is_err() {
            return Err(ListenFailed)
        }
        let acceptor = acceptor_res.unwrap();

        Ok(SmtpServer {
            acceptor: acceptor
        })
    }

    pub fn run(&mut self) {
        for mut stream_res in self.acceptor.incoming() {
            spawn(proc() {
                // TODO: is there a better way to handle an error here?
                let mut stream = SmtpStream::new(stream_res.unwrap());
                let mut transaction = SmtpTransaction::new();

                // Send the opening welcome message.
                stream.write_line("220 rustastic.org").unwrap();

                // Forever, looooop over command lines and handle them.
                'main_loop: loop {
                    // Find the right handler.
                    // TODO: check the return value and return appropriate error message,
                    // ie "500 Command line too long".
                    let line = stream.read_line().unwrap();
                    for h in handlers.iter() {
                        // Check that the begining of the command matches an existing SMTP
                        // command. This could be something like "HELO " or "RCPT TO:".
                        if line.as_slice().starts_with(*h.ref0()) {
                            if h.ref1().contains(&transaction.state) {
                                // We're good to go!
                                (*h.ref2())(
                                    &mut stream,
                                    &mut transaction,
                                    line.as_slice().slice_from((*h.ref0()).len())
                                ).unwrap(); // TODO: avoid unwrap here.
                                continue 'main_loop;
                            } else {
                                // Bad sequence of commands.
                                stream.write_line("503 Bad sequence of commands").unwrap();
                                continue 'main_loop;
                            }
                        }
                    }
                    // No valid command was given.
                    stream.write_line("500 Command unrecognized").unwrap();
                }
            });
        }
    }
}

fn handle_command_helo(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    if line.len() == 0 || utils::get_domain_len(line) != line.len() {
        stream.write_line("501 Domain name is invalid").unwrap();
        Ok(())
    } else {
        transaction.domain = Some(line.into_string());
        transaction.state = Helo;
        stream.write_line("250 OK").unwrap();
        Ok(())
    }
}

fn handle_command_mail(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    if line.char_at(0) != '<' || line.char_at(line.len() - 1) != '>' {
        stream.write_line("501 Email address invalid, must start with < and end with >").unwrap();
        Ok(())
    } else {
        let mailbox_res = Mailbox::parse(line.slice(1, line.len() - 1));
        match mailbox_res {
            Err(err) => {
                stream.write_line(format!("501 Email address invalid: {}", err).as_slice())
                    .unwrap();
            },
            Ok(mailbox) => {
                transaction.from = Some(mailbox);
                transaction.state = Mail;
                stream.write_line("250 OK").unwrap();
            }
        }
        Ok(())
    }
}

fn handle_command_rcpt(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    if transaction.to.len() >= 100 {
        stream.write_line("452 Too many recipients").unwrap();
        Ok(())
    } else if line.char_at(0) != '<' || line.char_at(line.len() - 1) != '>' {
        stream.write_line("501 Email address invalid, must start with < and end with >").unwrap();
        Ok(())
    } else {
        let mailbox_res = Mailbox::parse(line.slice(1, line.len() - 1));
        match mailbox_res {
            Err(err) => {
                stream.write_line(format!("501 Email address invalid: {}", err).as_slice())
                    .unwrap();
            },
            Ok(mailbox) => {
                transaction.to.push(mailbox);
                transaction.state = Rcpt;
                stream.write_line("250 OK").unwrap();
            }
        }
        Ok(())
    }
}

fn handle_command_data(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    if line.len() != 0 {
        stream.write_line("501 No arguments allowed").unwrap();
    } else {
        stream.write_line("354 Start mail input; end with <CRLF>.<CRLF>").unwrap();
        transaction.data = Some(stream.read_data().unwrap());
        transaction.state = Data;
        // ... (here is where we'd handle the finished transaction)
        transaction.reset();
        stream.write_line("250 OK").unwrap();
    }
    Ok(())
}

fn handle_command_rset(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    if line.len() != 0 {
        stream.write_line("501 No arguments allowed").unwrap();
    } else {
        transaction.reset();
        stream.write_line("250 OK").unwrap();
    }
    Ok(())
}

#[allow(unused_variable)]
fn handle_command_vrfy(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    stream.write_line("252 Cannot VRFY user").unwrap();
    Ok(())
}

#[allow(unused_variable)]
fn handle_command_expn(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    stream.write_line("252 Cannot EXPN mailing list").unwrap();
    Ok(())
}

#[allow(unused_variable)]
fn handle_command_help(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    if line.len() == 0 || line.char_at(0) == ' ' {
        stream.write_line("502 Command not implemented").unwrap();
    } else {
        stream.write_line("500 Command unrecognized").unwrap();
    }
    Ok(())
}

#[allow(unused_variable)]
fn handle_command_noop(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    if line.len() == 0 || line.char_at(0) == ' ' {
        stream.write_line("250 OK").unwrap();
    } else {
        stream.write_line("500 Command unrecognized").unwrap();
    }
    Ok(())
}

#[allow(unused_variable)]
fn handle_command_quit(stream: &mut SmtpStream<TcpStream>,
                       transaction: &mut SmtpTransaction,
                       line: &str) -> Result<(), ()> {
    stream.write_line("221 rustastic.org").unwrap();
    Err(())
}
