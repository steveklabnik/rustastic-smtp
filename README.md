# Rustastic SMTP

Rustastic SMTP is meant to provide SMTP tools such as email address parsing
utilities as well as a configurable SMTP server and client.

The goal is to eventually comply with the
[SMTP spec from RFC 5321](http://tools.ietf.org/html/rfc5321).
If you would like to get involved, feel free to create an issue so we can discuss publicly and
iterate on ideas together.

# Example

```rust
extern crate rsmtp;

use rsmtp::server::{
    SmtpServer,
    SmtpServerConfig,
    SmtpServerEventHandler,
    SmtpTransaction
};

#[deriving(Clone)]
struct Handler;

impl SmtpServerEventHandler for Handler {
    fn handle_transaction(&mut self, transaction: &SmtpTransaction) -> Result<(), ()> {
        println!("Save to a database, send to an API, whatever you want :-)");
        Ok(())
    }
}

fn main() {
    let config = SmtpServerConfig {
        ip: "127.0.0.1",
        domain: "rustastic.org",
        port: 2525,
        max_recipients: 100,
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

This project is released under the terms of the MIT license. A copy of the license if available [here](LICENSE).
