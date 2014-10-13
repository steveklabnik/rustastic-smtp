# Rustastic SMTP

Rustastic SMTP is meant to provide SMTP tools such as email address parsing
utilities as well as a configurable SMTP server and client.

The goal is to eventually comply with the
[SMTP spec from RFC 5321](http://tools.ietf.org/html/rfc5321).
If you would like to get involved, feel free to create an issue so we can discuss publicly and
iterate on ideas together.

**This project is very much a work in progress. I'm planning on releasing a `v1 beta` at about the same time Rust reaches `v1` and stricly respect SemVer from there. Until then, I'll be making breaking changes from time to time.**

# Example

```rust
extern crate rsmtp;

use rsmtp::server::{SmtpServer, SmtpServerEventHandler, SmtpServerConfig};
use rsmtp::common::transaction::SmtpTransaction;
use rsmtp::common::mailbox::Mailbox;
use rsmtp::common::{
    MIN_ALLOWED_MESSAGE_SIZE,
    MIN_ALLOWED_LINE_SIZE,
    MIN_ALLOWED_RECIPIENTS
};

#[deriving(Clone)]
struct Handler;

impl SmtpServerEventHandler for Handler {
    fn handle_rcpt(&mut self, transaction: &SmtpTransaction, mailbox: &Mailbox) -> Result<(), ()> {
        println!("Check in a database if this recipient is valid and more if you want.");
        Ok(())
    }

    fn handle_transaction(&mut self, transaction: &SmtpTransaction) -> Result<(), ()> {
        println!("Save to a database, send to an API, whatever you want :-)");
        Ok(())
    }
}

fn main() {
    let config = SmtpServerConfig {
        ip: "0.0.0.0",
        domain: "rustastic.org",
        port: 25,
        max_recipients: MIN_ALLOWED_RECIPIENTS,
        max_message_size: MIN_ALLOWED_MESSAGE_SIZE,
        max_line_size: MIN_ALLOWED_LINE_SIZE,
        debug: true
    };
    let mut server = SmtpServer::new(config, Handler).unwrap();
    server.run();
}
```

There is also
[an example SMTP server](https://github.com/conradkleinespel/rustastic-smtp-test-server), so that
you can quickly see it running:
```shell
git clone https://github.com/conradkleinespel/rustastic-smtp-test-server.git
cd rustastic-smtp-test-server
cargo build
./target/smtp-test-server
```

# Documentation

Rustastic SMTP uses Rust's built-in documentation system.

You can build the latest documentation using [Cargo](http://crates.io/) like so:

```shell
git clone https://github.com/conradkleinespel/rustastic-smtp.git
cd rustastic-smtp
cargo doc
```

Then, open the file `target/doc/rsmtp/index.html` in your browser of choice.

# Running tests

This project is linked with [rust-ci](http://rust-ci.org/conradkleinespel/rustastic-smtp) where
you can see the latest build status.

If you would like to run the tests yourself, here's how to do that, using
[Cargo](http://crates.io/):

```shell
git clone https://github.com/conradkleinespel/rustastic-smtp.git
cd rustastic-smtp
cargo test
```

# License

Rustastic SMTP is distributed under the terms of the Apache License (Version 2.0).
See [LICENSE.txt](LICENSE.txt) for more information.
