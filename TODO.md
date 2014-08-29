# Things to do

There are a bunch of things to do. It is true that not all of them are fun, but all of them are
useful and will need to be done at some point or another.

If you are working on this or want on this, please open an issue so that other people know you are on it. This way, we don't all do the same stuff.

## Things that are needed now

* More tests.
* Switching the utils to using `Option` instead of `return 0` to convey the absence of something.
* Remove calls to `unwrap` and actually handle errors.
* Enable handling received mails with a configurable driver.
* Support Ipv4 and Ipv6 address literals in commands.
* Allow empty reverse path, aka `<>`, in `MAIL` command.
* Enforce configurable limits on `DATA` size and number of recipients.
* Enable configuration of timeouts.
* Actually read data from the client, not dummy data like it is now.

## Things worth discussing but needed only later

* ESMTP support and most common extensions.
* More mail handling drivers (PostgreSQL, MySql, etc).
* Email body parsing.
* Allow mail relaying.

Other ideas? Let us know the issues :-)
