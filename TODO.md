# Things to do

There are a bunch of things to do. It is true that not all of them are fun, but all of them are
useful and will need to be done at some point or another.

If you are working on this or want on this, please open an issue so that other people know you are on it. This way, we don't all do the same stuff.

## Things that are needed now

* Switching the utils to using `Option` instead of `return 0` to convey the absence of something. Make unsafe functions `unsafe`.
* Support for timeout configuration. See: https://github.com/rust-lang/rust/issues/15802.
* Log errors instead of just calling `unwrap`. Log file? `write` thread safe?
* Add "Received" and "Return-Path" headers once the message is saved to a file.
* Make handler receive the body as kind of a stream, to avoid having the entire body in memory.
* Tests
	* `SmtpStream` errors.
	* `SmtpServer*`: any ideas on how to test it?

## Things worth discussing but needed only later

* Handling of mail with a fixed size threadpool?
* Extension system:
    * Allowed states
    * Add commands
    * Add args to `MAIL`
    * Add args to `RCPT`
    * Increase command line length
    * Increase text line length
    * Disallow commands under certain conditions
    * Add states
* Allow mail relaying.
* Implement EXPN & VRFY.

## Other ideas

* Make safe (return `Option`) public versions of `MailboxLocalPart` functions.
* Allow configuring via a configuration file to avoid recompilation.
* Built in mail handling drivers (PostgreSQL, MySql, etc).

Other ideas? Let us know the issues :-)
