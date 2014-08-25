use std::io::net::tcp::{TcpListener, TcpAcceptor, TcpStream};
use std::io::{Listener, Acceptor};
use super::reader::{SmtpReader};

pub struct SmtpServer {
    acceptor: TcpAcceptor
}

#[deriving(Show)]
pub enum SmtpServerError {
    BindFailed,
    ListenFailed
}

struct SmtpTransaction {
    reader: SmtpReader<TcpStream>
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
                let mut stream = stream_res.unwrap();
                SmtpServer::handle_client(&mut stream);
            });
        }
    }

    fn handle_client(stream: &mut TcpStream) {
        let mut reader = SmtpReader::new(stream.clone());
        loop {
            let line = reader.read_line();
            match line {
                Ok(s) => println!("command: {}", s),
                Err(e) => fail!("{}", e)
            }
        }
    }
}

#[test]
fn test_server() {
    // ...
}
