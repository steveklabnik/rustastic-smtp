# Things to do

There are a bunch of things to do. It is true that not all of them are fun, but all of them are
useful and will need to be done at some point or another.

If you are working on this or want on this, please open an issue so that other people know you are on it. This way, we don't all do the same stuff.

## Things that are needed now

* Switching the utils to using `Option` instead of `return 0` to convey the absence of something.
* Remove calls to `unwrap` and actually handle errors.
* Enable handling received mails with a configurable driver.
* Allow empty reverse path, aka `<>`, in `MAIL` command.
* Enforce configurable limits on `DATA` size and number of recipients.
* Enable configuration of timeouts.
* Documentation.
* More tests.
* Case insensitive command name matching.
* Make MailboxLocalPart public and make constructors returns options rather than assuming arguments are valid.

## Things worth discussing but needed only later

* ESMTP support and most common extensions.
* More mail handling drivers (PostgreSQL, MySql, etc).
* Email body parsing.
* Allow mail relaying.
* Implement EXPN & VRFY.
* Make commands optional via configuration.

## Other ideas

* Make stream readers & writers faster.

Other ideas? Let us know the issues :-)
