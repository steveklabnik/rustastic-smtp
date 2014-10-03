// Copyright 2014 The Rustastic SMTP Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Utility functions used in SMTP clients and SMTP servers.

/// Returns a completely unescaped version of a quoted string.
///
/// This is useful for showing the email to a human, as it is easier to read.
pub fn unescape_quoted_string(s: &str) -> String {
    let mut i = 1u; // start after the opening quote
    let mut out = String::with_capacity(s.len());

    // don't go until the end, since the last char is the closing quote
    while i < s.len() - 1 {
        if is_atext(s.char_at(i)) || is_qtext_smtp(s.char_at(i)) {
            out.push(s.char_at(i));
            i += 1;
        } else {
            out.push(s.char_at(i + 1));
            i += 2;
        }
    }

    out
}

#[test]
fn test_unescape_quoted_string() {
    assert_eq!("b{}\"la.bla", unescape_quoted_string("\"b{}\\\"la\\.\\bla\"").as_slice());
    assert_eq!("", unescape_quoted_string("\"\"").as_slice());
    assert_eq!("a\\", unescape_quoted_string("\"a\\\\\"").as_slice());
}

/// Returns a simplified version of a quoted string. This can be another
/// quoted string or a dot string.
///
/// This is useful for showing the email to a human, as it is easier to read.
pub fn simplify_quoted_string(s: &str) -> String {
    let mut out = unescape_quoted_string(s);

    // If we have a valid dot-string, return that.
    if get_dot_string_len(out.as_slice()) == out.len() {
        return out;
    }

    // If we don't have a dot-string, remove useless escape sequences.
    out = String::with_capacity(s.len());
    out.push('"');
    let mut i = 1u; // Start after the opening quote.
    while i < s.len() - 1 { // End before the closing quote.
        // If we have a regular char, add it.
        if is_qtext_smtp(s.char_at(i)) {
            out.push(s.char_at(i));
            i += 1;

        // If we have an escape sequence, check if it is useful or not.
        } else {
            if s.char_at(i + 1) == '"' || s.char_at(i + 1) == '\\' {
                out.push(s.char_at(i));
                out.push(s.char_at(i + 1));
                i += 2;
            } else {
                out.push(s.char_at(i + 1));
                i += 2;
            }
        }
    }
    out.push('"');

    out
}

#[test]
fn test_simplify_quoted_string() {
    assert_eq!("\"b{}\\\"la.bla\"", simplify_quoted_string("\"b{}\\\"la\\.\\bla\"").as_slice());
    assert_eq!("", simplify_quoted_string("\"\"").as_slice());
    assert_eq!("\"a\\\\\"", simplify_quoted_string("\"a\\\\\"").as_slice());
    assert_eq!("a{", simplify_quoted_string("\"a\\{\"").as_slice());
}

/// Returns the length of the longest subdomain found at the beginning
/// of the passed string.
///
/// A subdomain is as described
/// [in RFC 5321](http://tools.ietf.org/html/rfc5321#section-4.1.2).
pub fn get_subdomain_len(s: &str) -> uint {
    let mut i = 0u;
    let mut confirmed_min = 0u;
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
pub fn get_domain_len(s: &str) -> uint {
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
pub fn get_atom_len(s: &str) -> uint {
    let mut len = 0u;
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
pub fn get_dot_string_len(s: &str) -> uint {
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
pub fn is_atext(c: char) -> bool {
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
pub fn is_alnum(c: char) -> bool {
    match c {
        'A' .. 'Z' | 'a' .. 'z' | '0' .. '9' => true,
        _ => false
    }
}

#[test]
fn test_is_alnum() {
    let mut c = 0;
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
pub fn get_quoted_string_len(s: &str) -> uint {
    // We need at least "".
    if s.len() < 2 || s.char_at(0) != '"' {
        return 0
    }
    // Length of 1 since we have the opening quote.
    let mut len = 1u;
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
pub fn is_qtext_smtp(c: char) -> bool {
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
pub fn is_quoted_pair_smtp(c1: char, c2: char) -> bool {
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
pub fn get_at_domain_len(s: &str) -> uint {
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
pub fn get_source_route_len(s: &str) -> uint {
    // The total length we have found for source routes.
    let mut len = 0u;

    // The length of the source route currently being checked in loop.
    let mut curr_len;

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

/// If the string starts with an ipv6 as present in email addresses, ie `[Ipv6:...]`, get its
/// length. Else return `0`.
pub fn get_possible_ipv6_len(ip: &str) -> uint {
    if ip.len() < 7 || ip.slice_to(6) != "[Ipv6:" {
        0
    } else {
        let mut i = 6u;
        while i < ip.len() && ip.char_at(i) != ']' {
            i += 1;
        }
        if i < ip.len() && ip.char_at(i) == ']' {
            i + 1
        } else {
            0
        }
    }
}

#[test]
fn test_get_possible_ipv6_len() {
    assert_eq!(10, get_possible_ipv6_len("[Ipv6:434]"));
    assert_eq!(10, get_possible_ipv6_len("[Ipv6:434][]"));
    assert_eq!(0, get_possible_ipv6_len("[Ipv6:434"));
    assert_eq!(7, get_possible_ipv6_len("[Ipv6:]"));
    assert_eq!(7, get_possible_ipv6_len("[Ipv6:]a"));
    assert_eq!(0, get_possible_ipv6_len("[Ipv"));
}

/// If the string starts with an ipv4 as present in email addresses, ie `[...]`, get its
/// length. Else return `0`.
pub fn get_possible_ipv4_len(ip: &str) -> uint {
    if ip.len() < 3 || ip.char_at(0) != '[' || ip.char_at(1) > '9' || ip.char_at(1) < '0' {
        0
    } else {
        let mut i = 1u;
        while i < ip.len() && ip.char_at(i) != ']' {
            i += 1;
        }
        if i < ip.len() && ip.char_at(i) == ']' {
            i + 1
        } else {
            0
        }
    }
}

#[test]
fn test_get_possible_ipv4_len() {
    assert_eq!(0, get_possible_ipv4_len("[Ipv6:]"));
    assert_eq!(3, get_possible_ipv4_len("[1]"));
    assert_eq!(3, get_possible_ipv4_len("[1]1"));
    assert_eq!(0, get_possible_ipv4_len("[]"));
}
