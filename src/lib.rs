//! Rustastic SMTP is meant to provide SMTP tools such as email parsing
//! utilities, an SMTP server, an SMTP client... At the moment, the library is
//! barely usable and partly undocumented. The goal is to eventually comply with
//! the [SMTP spec from RFC 5321](http://tools.ietf.org/html/rfc5321).

extern crate libc;

pub mod stream;
pub mod mailbox;
pub mod server;

/*
fn main() {
    let mut server = server::SmtpServer::new().unwrap();
    server.run();
}
// */
