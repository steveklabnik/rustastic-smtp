use std::string::{String};
use super::{utils};

/// Maximum length of the local part.
static MAX_MAILBOX_LOCAL_PART_LEN: uint = 64;

/// Maximum length of an email address.
///
/// The RFC doesn't actually specify 254 chars, but it does say that a reverse path starts with
/// "<", ends with ">" and including those symbols has a maximum length of 256.
static MAX_MAILBOX_LEN: uint = 254;

/// Maximum length of a domain name.
static MAX_DOMAIN_LEN: uint = 255;

#[test]
fn test_static_vars() {
    assert_eq!(64, MAX_MAILBOX_LOCAL_PART_LEN);
    assert_eq!(254, MAX_MAILBOX_LEN);
    assert_eq!(255, MAX_DOMAIN_LEN);
}

/// Represents the local part of an email address, aka the username.
#[deriving(PartialEq, Eq, Clone, Show)]
struct MailboxLocalPart {
    /// This is a version of the local part for use in the SMTP protocol.
    ///
    /// This is either a dot-string or a quoted-string, whatever is shortest as
    /// recommended in RFC 5321.
    smtp_string: String,
    /// This is a version of the local part that is completely unescaped.
    ///
    /// It is human readable but not suitable for use in SMTP.
    human_string: String
}

impl MailboxLocalPart {
    /// Create a local part from a dot-string.
    fn from_dot_string(dot_string: &str) -> MailboxLocalPart {
        MailboxLocalPart {
            human_string: dot_string.into_string(),
            smtp_string: dot_string.into_string()
        }
    }

    /// Create a local part from a quoted-string.
    ///
    /// Since a quoted-string can sometimes be simplified, this function tries to simplify it
    /// as much as possible.
    fn from_quoted_string(quoted_string: &str) -> MailboxLocalPart {
        MailboxLocalPart {
            human_string: utils::unescape_quoted_string(quoted_string),
            smtp_string: utils::simplify_quoted_string(quoted_string)
        }
    }
}

#[test]
fn test_local_part() {
    let mut lp1: MailboxLocalPart;
    let mut lp2: MailboxLocalPart;
    let mut lp3: MailboxLocalPart;
    let mut lp4: MailboxLocalPart;
    let mut lp5: MailboxLocalPart;

    lp1 = MailboxLocalPart::from_dot_string("rust.cool");
    lp2 = MailboxLocalPart::from_quoted_string("\"rust \\a cool\"");
    lp3 = MailboxLocalPart::from_quoted_string("\"rust.cool\"");
    lp4 = MailboxLocalPart::from_quoted_string("\"rust.cool.\"");
    lp5 = MailboxLocalPart::from_quoted_string("\"rust\\\\\\b\\;.c\\\"ool\"");

    assert!(lp1.clone() == lp1.clone());
    assert!(lp2.clone() == lp2.clone());
    assert!(lp1.clone() != lp2.clone());

    assert_eq!(lp1.smtp_string.as_slice(), "rust.cool");
    assert_eq!(lp1.human_string.as_slice(), "rust.cool");

    assert_eq!(lp2.smtp_string.as_slice(), "\"rust a cool\"");
    assert_eq!(lp2.human_string.as_slice(), "rust a cool");

    assert_eq!(lp3.smtp_string.as_slice(), "rust.cool");
    assert_eq!(lp3.human_string.as_slice(), "rust.cool");

    assert_eq!(lp4.smtp_string.as_slice(), "\"rust.cool.\"");
    assert_eq!(lp4.human_string.as_slice(), "rust.cool.");

    assert_eq!(lp5.smtp_string.as_slice(), "\"rust\\\\b;.c\\\"ool\"");
    assert_eq!(lp5.human_string.as_slice(), "rust\\b;.c\"ool");
}

/// Represents the foreign part of an email address, aka the host.
#[deriving(PartialEq, Eq, Clone, Show)]
enum MailboxForeignPart {
    Domain(String),
    Ipv4Addr(u8, u8, u8, u8),
    Ipv6Addr(u16, u16, u16, u16, u16, u16, u16, u16)
}

#[test]
fn test_foreign_part() {
    let domain_text = "rustastic.org";
    let domain = Domain(domain_text.into_string());
    let ipv4_1 = Ipv4Addr(127, 0, 0, 1);
    let ipv4_2 = Ipv4Addr(192, 168, 21, 21);
    let ipv6_1 = Ipv6Addr(1, 2, 3, 4, 5, 6, 7, 8);
    let ipv6_2 = Ipv6Addr(8, 7, 6, 5, 4, 3, 2, 1);

    assert!(domain == domain);
    assert!(domain != Domain(domain_text.into_string() + "bullshit"));
    assert!(domain != ipv4_1);
    assert!(domain != ipv6_1);

    assert!(ipv4_1.clone() == ipv4_1.clone());
    assert!(ipv4_1.clone() != ipv4_2.clone());
    assert!(ipv4_1 != ipv6_1);
    assert!(ipv4_1 != domain);

    assert!(ipv6_1.clone() == ipv6_1.clone());
    assert!(ipv6_1.clone() != ipv6_2.clone());
    assert!(ipv6_1 != ipv4_1);
    assert!(ipv6_1 != domain);
}

/// Represents an email address, aka "mailbox" in the SMTP spec.
///
/// It is composed of a local part and a foreign part.
#[deriving(PartialEq, Eq, Clone, Show)]
pub struct Mailbox {
    local_part: MailboxLocalPart,
    foreign_part: MailboxForeignPart
}

/// Represents an error that occured while trying to parse an email address.
#[deriving(PartialEq, Eq, Show)]
pub enum MailboxParseError {
    /// The maximum length of 64 octets [as per RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.5.3.1.1) is exceeded.
    LocalPartTooLong,
    /// The local part was neither a atom, nor a quoted string.
    LocalPartUnrecognized,
    /// The foreign part was neither a domain, nor an IP.
    ForeignPartUnrecognized,
    /// The maximum length of 255 octets [as per RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.5.3.1.2) is exceeded.
    DomainTooLong,
    /// The maximum length of 254 octets (256 - 2 for punctuaction) [as per RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.5.3.1.3) is exceeded.
    TooLong,
    /// If no @ was present.
    AtNotFound
}

impl Mailbox {
    /// Creates a `Mailbox` from a string if the string contains a valid email
    /// address. Otherwise, returns a `MailboxParseError`.
    ///
    /// The argument should be of the form:
    /// `hello@world.com`
    /// This function does *not* expect anything to wrap the passed email
    /// address. For example, this will result in an error:
    /// `<hello@world.com>`
    pub fn parse(s: &str) -> Result<Mailbox, MailboxParseError> {
        let mut local_part: MailboxLocalPart;
        let mut foreign_part: MailboxForeignPart;

        // Skip the source routes as specified in RFC 5321.
        let mut offset: uint = utils::get_source_route_len(s);

        // Get the local part.
        let dot_string_len = utils::get_dot_string_len(s.slice_from(offset));
        if dot_string_len > 0 {
            if dot_string_len > MAX_MAILBOX_LOCAL_PART_LEN {
                return Err(LocalPartTooLong);
            }
            local_part = MailboxLocalPart::from_dot_string(
                s.slice(offset, offset + dot_string_len)
            );
            offset += dot_string_len;
        } else {
            let quoted_string_len = utils::get_quoted_string_len(s.slice_from(offset));
            if quoted_string_len == 0 {
                return Err(LocalPartUnrecognized);
            }
            if quoted_string_len > MAX_MAILBOX_LOCAL_PART_LEN {
                return Err(LocalPartTooLong);
            }
            local_part = MailboxLocalPart::from_quoted_string(
                s.slice(offset, offset + quoted_string_len)
            );
            offset += quoted_string_len;
        }

        // Check if the email address continues to find an @.
        if offset >= s.len() {
            return Err(AtNotFound);
        }
        // If no @ is found, it means we're still in what should be the local
        // part but it is invalid, ie "rust is@rustastic.org".
        if s.char_at(offset) != '@' {
            return Err(LocalPartUnrecognized);
        }
        offset += 1;

        let domain_len = utils::get_domain_len(s.slice_from(offset));
        // Do we have no valid domain ?
        if domain_len == 0 {
            return Err(ForeignPartUnrecognized);
        }
        // Is the domain is too long ?
        if domain_len > MAX_DOMAIN_LEN {
            return Err(DomainTooLong);
        }

        // Save the domain.
        foreign_part = Domain(
            s.slice(offset, offset + domain_len).into_string()
        );
        offset += domain_len;

        // Example would be "rust.is@rustastic.org{}" where "rustastic.org{}"
        // would be considered an invalid domain name.
        if offset != s.len() {
            Err(ForeignPartUnrecognized)
        // Overall, is the email address to long? We could test this at the
        // beginning of the function to potentially save processing power, but
        // this shouldn't happen too often and this error doesn't give much
        // information whereas LocalPartTooLong is more precise which allows
        // for more understandable debug messages.
        } else if offset > MAX_MAILBOX_LEN {
            Err(TooLong)
        } else {
            Ok(Mailbox {
                local_part: local_part,
                foreign_part: foreign_part
            })
        }
    }
}

#[test]
fn test_mailbox() {
    let path_1 = Mailbox::parse("rust.is@rustastic.org").unwrap();
    let path_2 = Mailbox::parse("rust.is.not@rustastic.org").unwrap();

    assert!(path_1 == path_1.clone());
    assert!(path_2 == path_2.clone());
    assert!(path_1 != path_2);

    let path_3 = Mailbox::parse("\"hello\"@rust").unwrap();
    assert_eq!(path_3.local_part.smtp_string.as_slice(), "hello");
    assert_eq!(path_3.local_part.human_string.as_slice(), "hello");
    assert_eq!(path_3.foreign_part, Domain("rust".into_string()));

    Mailbox::parse(
        String::from_char(MAX_MAILBOX_LOCAL_PART_LEN, 'a')
            .append("@t.com")
            .as_slice()
    ).unwrap();
    assert_eq!(Err(LocalPartTooLong), Mailbox::parse(
        String::from_char(MAX_MAILBOX_LOCAL_PART_LEN + 1, 'a')
            .append("@t.com")
            .as_slice()
    ));
    assert_eq!(Err(LocalPartUnrecognized), Mailbox::parse("t @t.com{"));
    assert_eq!(Err(LocalPartUnrecognized), Mailbox::parse("t "));
    assert_eq!(Err(ForeignPartUnrecognized), Mailbox::parse("t@{}"));
    assert_eq!(Err(ForeignPartUnrecognized), Mailbox::parse("t@t.com{"));
    // The check here is to expect something else than DomainTooLong.
    assert_eq!(Err(TooLong), Mailbox::parse(
        ("rust@".into_string() + String::from_char(MAX_DOMAIN_LEN, 'a'))
            .as_slice()
    ));
    assert_eq!(Err(DomainTooLong), Mailbox::parse(
        ("rust@".into_string() + String::from_char(MAX_DOMAIN_LEN + 1, 'a'))
            .as_slice()
    ));
    Mailbox::parse(
        ("rust@".into_string() + String::from_char(MAX_MAILBOX_LEN - 5, 'a'))
            .as_slice()
    ).unwrap();
    assert_eq!(Err(TooLong), Mailbox::parse(
        ("rust@".into_string() + String::from_char(MAX_MAILBOX_LEN - 4, 'a'))
            .as_slice()
    ));
    assert_eq!(Err(AtNotFound), Mailbox::parse("t"));
}
