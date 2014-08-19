//! Rustastic SMTP is meant to provide SMTP tools such as email parsing
//! utilities, an SMTP server, an SMTP client... At the moment, the library is
//! barely usable and partly undocumented. The goal is to eventually comply with
//! the [SMTP spec from RFC 5321](http://tools.ietf.org/html/rfc5321).

use std::string::{String};

static MAX_MAILBOX_LOCAL_PART_LEN: uint = 64;
static MAX_MAILBOX_LEN: uint = 254;
static MAX_DOMAIN_LEN: uint = 255;

/// Represents the foreign part of an email address, aka the host.
#[deriving(PartialEq, Eq, Clone, Show)]
enum MailboxForeignPart {
    Domain(String),
    Ipv4Addr(u8, u8, u8, u8),
    Ipv6Addr(u16, u16, u16, u16, u16, u16, u16, u16)
}

/// Represents the local part of an email address, aka the username.
#[deriving(PartialEq, Eq, Clone, Show)]
struct MailboxLocalPart {
    // This is a version of the local part for use in the SMTP protocol. This is
    // either a dot-string or a quoted-string, whatever is shortest as
    // recommended in RFC 5321.
    smtp_string: String,
    // This is a version of the local part that is completely unescaped. It is
    // human readable but not suitable for use in SMTP.
    human_string: String
}

impl MailboxLocalPart {
    fn from_dot_string(s: &str) -> MailboxLocalPart {
        MailboxLocalPart {
            human_string: s.into_string(),
            smtp_string: s.into_string()
        }
    }

    fn from_quoted_string(s: &str) -> MailboxLocalPart {
        MailboxLocalPart {
            human_string: unescape_quoted_string(s),
            smtp_string: simplify_quoted_string(s)
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

/// Returns a completely unescaped version of a quoted string.
///
/// This is useful for showing the email to a human, as it is easier to read.
fn unescape_quoted_string(s: &str) -> String {
    let mut i: uint = 1; // start after the opening quote
    let mut out = String::with_capacity(s.len());

    // don't go until the end, since the last char is the closing quote
    while i < s.len() - 1 {
        if is_atext(s.char_at(i)) || is_qtext_smtp(s.char_at(i)) {
            out.push_char(s.char_at(i));
            i += 1;
        } else {
            out.push_char(s.char_at(i + 1));
            i += 2;
        }
    }

    out
}

/// Returns a simplified version of a quoted string. This can be another
/// quoted string or a dot string.
///
/// This is useful for showing the email to a human, as it is easier to read.
fn simplify_quoted_string(s: &str) -> String {
    let mut out = unescape_quoted_string(s);

    // If we have a valid dot-string, return that.
    if get_dot_string_len(out.as_slice()) == out.len() {
        return out;
    }

    // If we don't have a dot-string, remove useless escape sequences.
    out = String::with_capacity(s.len());
    out.push_char('"');
    let mut i: uint = 1; // Start after the opening quote.
    while i < s.len() - 1 { // End before the closing quote.
        // If we have a regular char, add it.
        if is_qtext_smtp(s.char_at(i)) {
            out.push_char(s.char_at(i));
            i += 1;

        // If we have an escape sequence, check if it is useful or not.
        } else {
            if s.char_at(i + 1) == '"' || s.char_at(i + 1) == '\\' {
                out.push_char(s.char_at(i));
                out.push_char(s.char_at(i + 1));
                i += 2;
            } else {
                out.push_char(s.char_at(i + 1));
                i += 2;
            }
        }
    }
    out.push_char('"');

    out
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
        let mut offset: uint = get_source_route_len(s);

        // Get the local part.
        let dot_string_len = get_dot_string_len(s.slice_from(offset));
        if dot_string_len > 0 {
            if dot_string_len > MAX_MAILBOX_LOCAL_PART_LEN {
                return Err(LocalPartTooLong);
            }
            local_part = MailboxLocalPart::from_dot_string(
                s.slice(offset, offset + dot_string_len)
            );
            offset += dot_string_len;
        } else {
            let quoted_string_len = get_quoted_string_len(s.slice_from(offset));
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

        let domain_len = get_domain_len(s.slice_from(offset));
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

/// Returns the length of the longest subdomain found at the beginning
/// of the passed string.
///
/// A subdomain is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
fn get_subdomain_len(s: &str) -> uint {
    let mut i: uint = 0;
    let mut confirmed_min: uint = 0;
    if s.len() == 0 {
        return 0
    }
    if is_alnum(s.char_at(0)) {
        i += 1;
        confirmed_min = i;
        while i < s.len() {
            if is_alnum(s.char_at(i)) {
                i += 1;
                confirmed_min = i;
            } else if s.char_at(i) == '-' {
                while i < s.len() && s.char_at(i) == '-' {
                    i += 1;
                }
            } else {
                break;
            }
        }
    }
    confirmed_min
}

#[test]
fn test_get_subdomain_len() {
    // Allow alnum and dashes in the middle, no points.
    assert_eq!(11, get_subdomain_len("helZo-4-you&&&"));
    assert_eq!(11, get_subdomain_len("hePRo-4-you.abc"));

    // Test with no content at the end.
    assert_eq!(10, get_subdomain_len("5---a-U-65"));
    assert_eq!(0, get_subdomain_len(""));

    // Disallow dash at the end.
    assert_eq!(5, get_subdomain_len("heS1o-&&&"));
    assert_eq!(0, get_subdomain_len("-hello-world"));
}

/// Returns the length of the longest domain found at the beginning of
/// the passed string.
///
/// A domain is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
fn get_domain_len(s: &str) -> uint {
    // We don't need to check if s.len() == 0 since get_subdomain_len(s)
    // already does it.
    let mut confirmed_min = get_subdomain_len(s);
    if confirmed_min > 0 {
        while confirmed_min < s.len() && s.char_at(confirmed_min) == '.' {
            let len = get_subdomain_len(s.slice_from(confirmed_min + 1));
            if len > 0 {
                confirmed_min += 1 + len;
            } else {
                break;
            }
        }
    }
    confirmed_min
}

#[test]
fn test_get_domain_len() {
    // Invalid domain.
    assert_eq!(0, get_domain_len(".hello"));
    assert_eq!(0, get_domain_len(""));
    assert_eq!(0, get_domain_len("----"));

    // Valid domains with dots and dashes.
    assert_eq!(18, get_domain_len("hello-rust.is.N1C3"));
    assert_eq!(18, get_domain_len("hello-rust.is.N1C3."));
    assert_eq!(18, get_domain_len("hello-rust.is.N1C3-"));
    assert_eq!(18, get_domain_len("hello-rust.is.N1C3-."));
    assert_eq!(18, get_domain_len("hello-rust.is.N1C3-&"));
    assert_eq!(18, get_domain_len("hello-rust.is.N1C3.&"));

    // Valid domains without dashes.
    assert_eq!(9, get_domain_len("hello.bla."));

    // Valid domains without dots.
    assert_eq!(9, get_domain_len("hello-bla."));
}

/// Returns the length of the longest atom found at the beginning of
/// the passed string.
///
/// An atom is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
fn get_atom_len(s: &str) -> uint {
    let mut len: uint = 0;
    while len < s.len() {
        if is_atext(s.char_at(len)) {
            len += 1
        } else {
            break;
        }
    }
    len
}

#[test]
fn test_get_atom_len() {
    assert_eq!(0, get_atom_len(" ---"));
    assert_eq!(4, get_atom_len("!a{`\\"));
    assert_eq!(4, get_atom_len("!a{`"));
    assert_eq!(0, get_atom_len(""));
}

/// Returns the length of the longest dot-string found at the beginning
/// of the passed string.
///
/// A dot-string is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
fn get_dot_string_len(s: &str) -> uint {
    // We don't need to check if s.len() == 0 since get_atom_len(s)
    // already does it.
    let mut confirmed_min = get_atom_len(s);
    if confirmed_min > 0 {
        while confirmed_min < s.len() && s.char_at(confirmed_min) == '.' {
            let len = get_atom_len(s.slice_from(confirmed_min + 1));
            if len > 0 {
                confirmed_min += 1 + len;
            } else {
                break;
            }
        }
    }
    confirmed_min
}

#[test]
fn test_get_dot_string_len() {
    assert_eq!(0, get_dot_string_len(""));
    assert_eq!(0, get_dot_string_len(" fwefwe"));
    assert_eq!(10, get_dot_string_len("-`-.bla.ok "));
    assert_eq!(10, get_dot_string_len("-`-.bla.ok"));
    assert_eq!(10, get_dot_string_len("-`-.bla.ok."));
}

/// Checks whether a character is valid `atext` as described
/// [in RFC 5322](http://tools.ietf.org/html/rfc5322#section-3.2.3).
fn is_atext(c: char) -> bool {
    match c {
        '!' | '#' | '$' | '%' | '&' | '\'' |
        '*' | '+' | '-' | '/' | '=' | '?'  |
        '^' | '_' | '`' | '{' | '|' | '}'  | '~' |
        'A' .. 'Z' | 'a' .. 'z' | '0' .. '9' => true,
        _ => false
    }
}

#[test]
fn test_is_atext() {
    // Valid atext.
    assert!(is_atext('!'));
    assert!(is_atext('#'));
    assert!(is_atext('$'));
    assert!(is_atext('%'));
    assert!(is_atext('&'));
    assert!(is_atext('\''));
    assert!(is_atext('*'));
    assert!(is_atext('+'));
    assert!(is_atext('-'));
    assert!(is_atext('/'));
    assert!(is_atext('='));
    assert!(is_atext('?'));
    assert!(is_atext('^'));
    assert!(is_atext('_'));
    assert!(is_atext('`'));
    assert!(is_atext('{'));
    assert!(is_atext('|'));
    assert!(is_atext('}'));
    assert!(is_atext('~'));
    assert!(is_atext('A'));
    assert!(is_atext('B'));
    assert!(is_atext('C'));
    assert!(is_atext('X'));
    assert!(is_atext('Y'));
    assert!(is_atext('Z'));
    assert!(is_atext('a'));
    assert!(is_atext('b'));
    assert!(is_atext('c'));
    assert!(is_atext('x'));
    assert!(is_atext('y'));
    assert!(is_atext('z'));
    assert!(is_atext('0'));
    assert!(is_atext('1'));
    assert!(is_atext('8'));
    assert!(is_atext('9'));

    // Invalid atext.
    assert!(!is_atext(' '));
    assert!(!is_atext('"'));
    assert!(!is_atext('('));
    assert!(!is_atext(')'));
    assert!(!is_atext(','));
    assert!(!is_atext('.'));
    assert!(!is_atext(':'));
    assert!(!is_atext(';'));
    assert!(!is_atext('<'));
    assert!(!is_atext('>'));
    assert!(!is_atext('@'));
    assert!(!is_atext('['));
    assert!(!is_atext(']'));
    assert!(!is_atext(127 as char));
}

/// Checks if a character is alphanumeric 7 bit ASCII.
fn is_alnum(c: char) -> bool {
    match c {
        'A' .. 'Z' | 'a' .. 'z' | '0' .. '9' => true,
        _ => false
    }
}

#[test]
fn test_is_alnum() {
    let mut c = 0 as u8;
    while c <= 127 {
        // Keep separate assertions for each range to get better error messages.
        if c >= 'A' as u8 && c <= 'Z' as u8 {
            assert!(is_alnum(c as char));
        } else if c >= 'a' as u8 && c <= 'z' as u8 {
            assert!(is_alnum(c as char));
        } else if c >= '0' as u8 && c <= '9' as u8 {
            assert!(is_alnum(c as char));
        } else {
            assert!(!is_alnum(c as char));
        }
        c += 1;
    }
}

/// Returns the length of the longest quoted-string found at the beginning of
/// the passed string. The length includes escaping backslashes and double
/// quotes.
///
/// A quoted-string is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
fn get_quoted_string_len(s: &str) -> uint {
    // We need at least "".
    if s.len() < 2 || s.char_at(0) != '"' {
        return 0
    }
    // Length of 1 since we have the opening quote.
    let mut len: uint = 1;
    loop {
        // Regular text.
        if len < s.len() && is_qtext_smtp(s.char_at(len)) {
            len += 1;
        // Escaped text.
        } else if len + 1 < s.len() &&
            is_quoted_pair_smtp(s.char_at(len), s.char_at(len + 1)) {
            len += 2;
        } else {
            break;
        }
    }
    if len < s.len() && s.char_at(len) == '"' {
        len + 1
    } else {
        0
    }
}

#[test]
fn test_get_quoted_string_len() {
    // Invalid.
    assert_eq!(0, get_quoted_string_len(""));
    assert_eq!(0, get_quoted_string_len(" "));
    assert_eq!(0, get_quoted_string_len("  "));
    assert_eq!(0, get_quoted_string_len(" \""));
    assert_eq!(0, get_quoted_string_len(" \" \""));
    assert_eq!(0, get_quoted_string_len("\""));
    assert_eq!(0, get_quoted_string_len("\"Rust{\\\\\\\"\\a}\\stic"));

    // Valid.
    assert_eq!(2, get_quoted_string_len("\"\""));
    assert_eq!(19, get_quoted_string_len("\"Rust{\\\\\\\"\\a}\\stic\""));
    assert_eq!(19, get_quoted_string_len("\"Rust{\\\\\\\"\\a}\\stic\" "));
}

/// Checks whether a character is valid `qtextSMTP` as described
/// [in RFC 5322](http://tools.ietf.org/html/rfc5322#section-3.2.3).
fn is_qtext_smtp(c: char) -> bool {
    match c as int {
        32 .. 33 | 35 .. 91 | 93 .. 126 => true,
        _ => false
    }
}

#[test]
fn test_is_qtext_smtp() {
    assert!(!is_qtext_smtp(31 as char));
    assert!(is_qtext_smtp(' '));
    assert!(is_qtext_smtp('!'));
    assert!(!is_qtext_smtp('"'));
    assert!(is_qtext_smtp('#'));
    assert!(is_qtext_smtp('$'));
    assert!(is_qtext_smtp('Z'));
    assert!(is_qtext_smtp('['));
    assert!(!is_qtext_smtp('\\'));
    assert!(is_qtext_smtp(']'));
    assert!(is_qtext_smtp('^'));
    assert!(is_qtext_smtp('}'));
    assert!(is_qtext_smtp('~'));
    assert!(!is_qtext_smtp(127 as char));
}

/// Checks if a pair of characters represent a `quoted-pairSMTP` as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2)
fn is_quoted_pair_smtp(c1: char, c2: char) -> bool {
    c1 as int == 92 && (c2 as int >= 32 && c2 as int <= 126)
}

#[test]
fn test_is_quoted_pair_smtp() {
    assert!(is_quoted_pair_smtp('\\', ' '));
    assert!(is_quoted_pair_smtp('\\', '!'));
    assert!(is_quoted_pair_smtp('\\', '}'));
    assert!(is_quoted_pair_smtp('\\', '~'));
    assert!(!is_quoted_pair_smtp(' ', ' '));
}

/// Returns the length of the longest at-domain found at the beginning of
/// the passed string.
///
/// An at-domain is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
fn get_at_domain_len(s: &str) -> uint {
    if s.len() < 1 || s.char_at(0) != '@' {
        return 0
    }
    let len = get_domain_len(s.slice_from(1));

    // If we found a valid domain, we return its length plus 1 for the @.
    if len > 0 {
        len + 1
    } else {
        0
    }
}

#[test]
fn test_get_at_domain_len() {
    assert_eq!(0, get_at_domain_len(""));
    assert_eq!(0, get_at_domain_len("@"));
    assert_eq!(0, get_at_domain_len("@@"));
    assert_eq!(5, get_at_domain_len("@rust"));
    assert_eq!(5, get_at_domain_len("@rust{}"));
    assert_eq!(14, get_at_domain_len("@rustastic.org"));
}

/// Returns the length of the source routes found at the beginning of
/// the passed string.
///
/// Source routes are as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
fn get_source_route_len(s: &str) -> uint {
    // The total length we have found for source routes.
    let mut len: uint = 0;

    // The length of the source route currently being checked in loop.
    let mut curr_len: uint;

    loop {
        // Get the current source route.
        curr_len = get_at_domain_len(s.slice_from(len));
        if curr_len > 0 {
            len += curr_len;
            // Check if another source route is coming, if not, stop looking
            // for more source routes.
            if len < s.len() && s.char_at(len) == ',' {
                len += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Expect the source route declaration to end with ':'.
    if len < s.len() && s.char_at(len) == ':' {
        len + 1
    } else {
        0
    }
}

#[test]
fn test_get_source_route_len() {
    // Invalid.
    assert_eq!(0, get_source_route_len(""));
    assert_eq!(0, get_source_route_len("@rust,"));
    assert_eq!(0, get_source_route_len("@rust"));
    assert_eq!(0, get_source_route_len("@,@:"));
    assert_eq!(0, get_source_route_len("@rust,@troll"));
    assert_eq!(0, get_source_route_len("@rust,@tro{ll:"));

    // Valid.
    assert_eq!(13, get_source_route_len("@rust,@troll:"));
    assert_eq!(16, get_source_route_len("@rust.is,@troll:"));
}
