use std::io::net::tcp::{TcpListener, TcpAcceptor, TcpStream};
use std::io::{Listener, Acceptor};
use super::stream::{SmtpStream};
use super::mailbox::{Mailbox, MailboxParseError};

static handlers: &'static [HandlerDescription] = &[
    ("HELO", "HELO ", &[Init], handle_command_helo)
];

type HandlerDescription = (&'static str, &'static str, &'static [SmtpTransactionState], HandlerFunction);
type HandlerFunction = fn(&mut SmtpStream<TcpStream>, &mut SmtpTransaction);

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
    to: Option<Vec<Mailbox>>,
    from: Option<Mailbox>,
    data: Option<String>,
    state: SmtpTransactionState
}

impl SmtpTransaction {
    pub fn new() -> SmtpTransaction {
        SmtpTransaction {
            domain: None,
            to: None,
            from: None,
            data: None,
            state: Init
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
                stream.write_line("220 rustastic.org");

                // Forever, looooop over command lines and handle them.
                'main_loop: loop {
                    // Find the right handler.
                    // TODO: check the return value and return appropriate error message,
                    // ie "500 Command line too long".
                    let line = stream.read_line().unwrap();
                    for h in handlers.iter() {
                        // Check that the begining of the command matches an existing SMTP
                        // command. This could be something like "HELO " or "RCPT TO:".
                        if line.as_slice().starts_with(*h.ref1()) {
                            if h.ref2().contains(&transaction.state) {
                                // We're good to go!
                                (*h.ref3())(&mut stream, &mut transaction);
                                continue 'main_loop;
                            } else {
                                // Bad sequence of commands.
                                stream.write_line("503 Bad sequence of commands");
                                continue 'main_loop;
                            }
                        }
                    }
                    // No valid command was given.
                    stream.write_line("500 Command unrecognized");
                }
            });
        }
    }
}

fn handle_command_helo(stream: &mut SmtpStream<TcpStream>, transaction: &mut SmtpTransaction) {
    ()
}

#[test]
fn test_server() {
    // ...
}
