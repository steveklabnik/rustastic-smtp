use std::io::net::tcp::{TcpListener, TcpAcceptor, TcpStream};
use std::io::{Listener, Acceptor};
use super::stream::{SmtpStream};
use super::mailbox::{Mailbox, MailboxParseError};

pub struct SmtpServer {
    acceptor: TcpAcceptor
}

#[deriving(Show)]
pub enum SmtpServerError {
    BindFailed,
    ListenFailed
}

pub enum SmtpTransactionState {
    Initial,
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
            state: Initial
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
                // It's OK to unwrap here since even if this fails, the server
                // will keep going: only this connection will be aborted.
                SmtpServer::handle_client(stream_res.unwrap());
            });
        }
    }

    fn handle_client(s: TcpStream) {
        let mut stream = SmtpStream::new(s);
        let mut transaction = SmtpTransaction::new();

        stream.write_line("220 rustastic.org");
        loop {
            let line = stream.read_line().unwrap();
            match transaction.state {
                Initial => {
                    if line.as_slice().starts_with("HELO ") {
                        transaction.state = Helo;
                        transaction.domain = Some(
                            line.as_slice().slice_from(5).into_string()
                        );
                    } else {
                        stream.write_line("503 Bad sequence of commands");
                    }
                    println!("received nothing yet");
                },
                Helo => {
                    println!("received helo");
                    transaction.state = Mail;
                },
                Mail => {
                    println!("received mail");
                    transaction.state = Rcpt;
                },
                Rcpt => {
                    println!("received rcpt");
                    transaction.state = Data;
                },
                Data => {
                    println!("received data");
                }
            }
        }
    }
}

#[test]
fn test_server() {
    // ...
}
