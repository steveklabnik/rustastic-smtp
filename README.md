# Rustastic SMTP

This library is meant to provide SMTP tools such as email parsing utilities, an SMTP server, an SMTP client... At the moment, the library is barely usable and partly undocumented. The goal is to eventually comply with the [SMTP spec from RFC 5321](http://tools.ietf.org/html/rfc5321).

If you would like to get involved, feel free to create an issue so we can discuss publicly and iterate on ideas together.

```rust
extern crate rsmtp;

fn main() {
    let mut server = server::SmtpServer::new().unwrap();
    server.run();
}
```

This project is licensed under the terms of the MIT license. A copy of the license if available [here](LICENSE).
