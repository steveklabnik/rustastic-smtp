# Things to do

There are a bunch of things to do. It is true that not all of them are fun, but all of them are
useful and will need to be done at some point or another.

If you are working on this or want on this, please open an issue so that other people know you are on it. This way, we don't all do the same stuff.

## Things that are needed now

* Support for timeout configuration. See: https://github.com/rust-lang/rust/issues/15802.
* Log errors instead of just calling `unwrap`. Log file? `write` thread safe?
* Add config checks to force limits to be spec compliant.
* Documentation.
* Tests
	* `SmtpStream` errors.
	* `SmtpServer*`: any ideas on how to test it?

## Things worth discussing but needed only later

* Handling of mail with a fixed size threadpool?
* ESMTP support and most common extensions.
* More mail handling drivers (PostgreSQL, MySql, etc).
* Allow mail relaying.
* Implement EXPN & VRFY.
* Make commands optional via configuration.
* Switching the utils to using `Option` instead of `return 0` to convey the absence of something.
* Make safe (return `Option`) public versions of `MailboxLocalPart` functions.
* Allow configuring via a configuration file to avoid recompilation.

## Other ideas

* Make stream readers & writers faster.

Other ideas? Let us know the issues :-)
