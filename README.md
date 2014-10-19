# Rustastic SMTP

Rustastic SMTP is meant to provide SMTP tools such as email address parsing
utilities as well as a configurable SMTP server and client.

The goal is to eventually comply with the
[SMTP spec from RFC 5321](http://tools.ietf.org/html/rfc5321).
If you would like to get involved, feel free to create an issue so we can discuss publicly and
iterate on ideas together.

**This project is very much a work in progress. I'm planning on releasing a `v1 beta` at about the same time Rust reaches `v1` and stricly respect SemVer from there. Until then, I'll be making breaking changes from time to time.**

# Example

To help you get started and showcase `rsmtp` in action, we have built an [example SMTP server](https://github.com/conradkleinespel/rustastic-smtp-test-server/blob/master/src/main.rs).

If you want to easily build and run the example SMTP server, you can do that by running the following commands in your terminal:
```shell
git clone https://github.com/conradkleinespel/rustastic-smtp-test-server.git
cd rustastic-smtp-test-server
cargo build
sudo ./target/smtp-test-server
```

Note that admin rights are needed to run the server because the default port to listen on is below 1024. If you change the
server's port in the `src/main.rs` file to something like `2525`, you should not need admin rights anymore.

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
