//! # 4. Data Formats
//!
//! IMAP4rev1 uses textual commands and responses.  Data in
//! IMAP4rev1 can be in one of several forms: atom, number, string,
//! parenthesized list, or NIL.  Note that a particular data item
//! may take more than one form; for example, a data item defined as
//! using "astring" syntax may be either an atom or a string.

use std::{
    borrow::{Borrow, Cow},
    convert::TryFrom,
    ops::Deref,
};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
#[cfg(feature = "serdex")]
use serde::{Deserialize, Serialize};

use crate::utils::indicators::{
    is_any_text_char_except_quoted_specials, is_astring_char, is_atom_char, is_char8, is_text_char,
};

// ## 4.1. Atom

/// An atom consists of one or more non-special characters.
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Atom<'a> {
    inner: Cow<'a, str>,
}

impl<'a> Atom<'a> {
    pub fn verify(value: &str) -> bool {
        !value.is_empty() && value.bytes().all(is_atom_char)
    }

    pub fn inner(&self) -> &Cow<'a, str> {
        &self.inner
    }

    pub unsafe fn new_unchecked(inner: Cow<'a, str>) -> Self {
        Self { inner }
    }
}

impl<'a> TryFrom<&'a str> for Atom<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if Self::verify(value) {
            Ok(Self {
                inner: Cow::Borrowed(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<String> for Atom<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::verify(&value) {
            Ok(Atom {
                inner: Cow::Owned(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> Deref for Atom<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.inner.borrow()
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtomExt<'a> {
    inner: Cow<'a, str>,
}

impl<'a> AtomExt<'a> {
    pub fn verify(value: &str) -> bool {
        !value.is_empty() && value.bytes().all(is_astring_char)
    }

    pub fn inner(&self) -> &Cow<'a, str> {
        &self.inner
    }

    pub unsafe fn new_unchecked(inner: Cow<'a, str>) -> Self {
        Self { inner }
    }
}

impl<'a> TryFrom<&'a str> for AtomExt<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if Self::verify(value) {
            Ok(Self {
                inner: Cow::Borrowed(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<String> for AtomExt<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::verify(&value) {
            Ok(Self {
                inner: Cow::Owned(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> Deref for AtomExt<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.inner.borrow()
    }
}

// ## 4.2. Number
//
// A number consists of one or more digit characters, and
// represents a numeric value.

// ## 4.3. String

/// A string is in one of two forms: either literal or quoted string.
///
/// The empty string is represented as either "" (a quoted string
/// with zero characters between double quotes) or as {0} followed
/// by CRLF (a literal with an octet count of 0).
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IString<'a> {
    /// A literal is a sequence of zero or more octets (including CR and
    /// LF), prefix-quoted with an octet count in the form of an open
    /// brace ("{"), the number of octets, close brace ("}"), and CRLF.
    /// In the case of literals transmitted from server to client, the
    /// CRLF is immediately followed by the octet data.  In the case of
    /// literals transmitted from client to server, the client MUST wait
    /// to receive a command continuation request (...) before sending
    /// the octet data (and the remainder of the command).
    ///
    /// Note: Even if the octet count is 0, a client transmitting a
    /// literal MUST wait to receive a command continuation request.
    ///
    Literal(Literal<'a>),
    /// The quoted string form is an alternative that avoids the overhead of
    /// processing a literal at the cost of limitations of characters which may be used.
    ///
    /// A quoted string is a sequence of zero or more 7-bit characters,
    /// excluding CR and LF, with double quote (<">) characters at each end.
    ///
    Quoted(Quoted<'a>),
}

impl<'a> TryFrom<&'a str> for IString<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, ()> {
        if let Ok(quoted) = Quoted::try_from(value) {
            return Ok(IString::Quoted(quoted));
        }

        if let Ok(literal) = Literal::try_from(value.as_bytes()) {
            return Ok(IString::Literal(literal));
        }

        Err(())
    }
}

impl<'a> TryFrom<String> for IString<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, ()> {
        if let Ok(quoted) = Quoted::try_from(value.clone()) {
            return Ok(IString::Quoted(quoted));
        }

        if let Ok(literal) = Literal::try_from(value.into_bytes()) {
            return Ok(IString::Literal(literal));
        }

        Err(())
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal<'a> {
    inner: Cow<'a, [u8]>,
}

impl<'a> Literal<'a> {
    pub fn verify(bytes: &[u8]) -> bool {
        bytes.iter().all(|b| is_char8(*b))
    }

    pub fn inner(&self) -> &Cow<'a, [u8]> {
        &self.inner
    }

    /// Create a literal from a byte sequence without checking
    /// that it to conforms to IMAP's literal specification.
    ///
    /// # Safety
    ///
    /// Call this function only when you are sure that the byte sequence
    /// is a valid literal, i.e., that it does not contain 0x00.
    pub unsafe fn new_unchecked(inner: Cow<'a, [u8]>) -> Self {
        Self { inner }
    }
}

impl<'a> TryFrom<&'a [u8]> for Literal<'a> {
    type Error = ();

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if Literal::verify(value) {
            Ok(Literal {
                inner: Cow::Borrowed(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<Vec<u8>> for Literal<'a> {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if Literal::verify(&value) {
            Ok(Literal {
                inner: Cow::Owned(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> Deref for Literal<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Quoted<'a> {
    inner: Cow<'a, str>,
}

impl<'a> Quoted<'a> {
    pub fn verify(value: &str) -> bool {
        value.chars().all(|c| c.is_ascii() && is_text_char(c as u8))
    }

    pub fn inner(&self) -> &Cow<'a, str> {
        &self.inner
    }

    /// Create a quoted from a string without checking
    /// that it to conforms to IMAP's quoted specification.
    ///
    /// # Safety
    ///
    /// Call this function only when you are sure that the str
    /// is a valid quoted.
    pub unsafe fn new_unchecked(value: Cow<'a, str>) -> Self {
        Self { inner: value }
    }
}

impl<'a> TryFrom<&'a str> for Quoted<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if Quoted::verify(value) {
            Ok(Quoted {
                inner: Cow::Borrowed(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<String> for Quoted<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Quoted::verify(&value) {
            Ok(Quoted {
                inner: Cow::Owned(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> Deref for Quoted<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// This wrapper is merely used for formatting.
// The inner value can be public.
pub struct NString<'a>(pub Option<IString<'a>>);

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AString<'a> {
    // `1*ATOM-CHAR` does not allow resp-specials, but `1*ASTRING-CHAR` does ... :-/
    Atom(AtomExt<'a>),   // 1*ASTRING-CHAR /
    String(IString<'a>), // string
}

impl<'a> TryFrom<&'a str> for AString<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, ()> {
        if let Ok(atom) = AtomExt::try_from(value) {
            Ok(AString::Atom(atom))
        } else if let Ok(string) = IString::try_from(value) {
            Ok(AString::String(string))
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<String> for AString<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, ()> {
        if let Ok(atom) = AtomExt::try_from(value.clone()) {
            Ok(AString::Atom(atom))
        } else if let Ok(string) = IString::try_from(value) {
            Ok(AString::String(string))
        } else {
            Err(())
        }
    }
}

// 4.3.1.  8-bit and Binary Strings
//
//    8-bit textual and binary mail is supported through the use of a
//    [MIME-IMB] content transfer encoding.  IMAP4rev1 implementations MAY
//    transmit 8-bit or multi-octet characters in literals, but SHOULD do
//    so only when the [CHARSET] is identified.
//
//    Although a BINARY body encoding is defined, unencoded binary strings
//    are not permitted.  A "binary string" is any string with NUL
//    characters.  Implementations MUST encode binary data into a textual
//    form, such as BASE64, before transmitting the data.  A string with an
//    excessive amount of CTL characters MAY also be considered to be
//    binary.

// 4.4.    Parenthesized List
//
//    Data structures are represented as a "parenthesized list"; a sequence
//    of data items, delimited by space, and bounded at each end by
//    parentheses.  A parenthesized list can contain other parenthesized
//    lists, using multiple levels of parentheses to indicate nesting.
//
//    The empty list is represented as () -- a parenthesized list with no
//    members.

/// 4.5. NIL
///
/// The special form "NIL" represents the non-existence of a particular
/// data item that is represented as a string or parenthesized list, as
/// distinct from the empty string "" or the empty parenthesized list ().
///
///  Note: NIL is never used for any data item which takes the
///  form of an atom.  For example, a mailbox name of "NIL" is a
///  mailbox named NIL as opposed to a non-existent mailbox
///  name.  This is because mailbox uses "astring" syntax which
///  is an atom or a string.  Conversely, an addr-name of NIL is
///  a non-existent personal name, because addr-name uses
///  "nstring" syntax which is NIL or a string, but never an
///  atom.

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Tag<'a> {
    inner: Cow<'a, str>,
}

impl<'a> Tag<'a> {
    pub fn verify(value: &str) -> bool {
        !value.is_empty() && value.bytes().all(|c| is_astring_char(c) && c != b'+')
    }

    pub fn inner(&self) -> &Cow<'a, str> {
        &self.inner
    }

    pub unsafe fn new_unchecked(inner: Cow<'a, str>) -> Self {
        Self { inner }
    }

    pub fn random() -> Self {
        let mut rng = thread_rng();
        let buffer = [0u8; 8].map(|_| rng.sample(Alphanumeric));

        unsafe { Self::new_unchecked(Cow::Owned(String::from_utf8_unchecked(buffer.to_vec()))) }
    }
}

impl<'a> TryFrom<&'a str> for Tag<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if Self::verify(value) {
            Ok(Self {
                inner: Cow::Borrowed(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<String> for Tag<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::verify(&value) {
            Ok(Self {
                inner: Cow::Owned(value),
            })
        } else {
            Err(())
        }
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Text<'a> {
    inner: Cow<'a, str>,
}

impl<'a> Text<'a> {
    pub fn verify(value: &str) -> bool {
        !value.is_empty() && value.bytes().all(is_text_char)
    }

    pub fn inner(&self) -> &Cow<'a, str> {
        &self.inner
    }

    pub unsafe fn new_unchecked(inner: Cow<'a, str>) -> Self {
        Self { inner }
    }
}

impl<'a> TryFrom<&'a str> for Text<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if Self::verify(value) {
            Ok(Self {
                inner: Cow::Borrowed(value),
            })
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<String> for Text<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::verify(&value) {
            Ok(Self {
                inner: Cow::Owned(value),
            })
        } else {
            Err(())
        }
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Copy, Debug, PartialEq, Eq, Hash, Clone)]
pub struct QuotedChar {
    inner: char,
}

impl QuotedChar {
    pub fn verify(input: char) -> bool {
        if input.is_ascii() {
            is_any_text_char_except_quoted_specials(input as u8) || input == '\\' || input == '"'
        } else {
            false
        }
    }

    pub fn inner(&self) -> &char {
        &self.inner
    }

    pub unsafe fn new_unchecked(inner: char) -> Self {
        Self { inner }
    }
}

impl TryFrom<char> for QuotedChar {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if Self::verify(value) {
            Ok(QuotedChar { inner: value })
        } else {
            Err(())
        }
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Charset<'a> {
    Atom(Atom<'a>),
    Quoted(Quoted<'a>),
}

impl<'a> TryFrom<&'a str> for Charset<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if let Ok(atom) = Atom::try_from(value) {
            Ok(Charset::Atom(atom))
        } else if let Ok(quoted) = Quoted::try_from(value) {
            Ok(Charset::Quoted(quoted))
        } else {
            Err(())
        }
    }
}

impl<'a> TryFrom<String> for Charset<'a> {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Ok(atom) = Atom::try_from(value.clone()) {
            Ok(Charset::Atom(atom))
        } else if let Ok(quoted) = Quoted::try_from(value) {
            Ok(Charset::Quoted(quoted))
        } else {
            Err(())
        }
    }
}

#[cfg_attr(feature = "serdex", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyVec<T> {
    inner: Vec<T>,
}

impl<T> NonEmptyVec<T> {
    pub unsafe fn new_unchecked(inner: Vec<T>) -> Self {
        Self { inner }
    }
}

impl<T> TryFrom<Vec<T>> for NonEmptyVec<T> {
    type Error = ();

    fn try_from(inner: Vec<T>) -> Result<Self, Self::Error> {
        if !inner.is_empty() {
            Ok(Self { inner })
        } else {
            Err(())
        }
    }
}

impl<T> Deref for NonEmptyVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod test {
    use std::{convert::TryInto, str::from_utf8};

    use super::*;
    use crate::codec::Encode;

    #[test]
    fn test_conversion() {
        assert_eq!(
            IString::try_from("AAA").unwrap(),
            IString::Quoted("AAA".try_into().unwrap()).into()
        );
        assert_eq!(
            IString::try_from("\"AAA").unwrap(),
            IString::Quoted("\"AAA".try_into().unwrap()).into()
        );

        assert_ne!(
            IString::try_from("\"AAA").unwrap(),
            IString::Quoted("\\\"AAA".try_into().unwrap()).into()
        );
    }

    #[test]
    fn test_charset() {
        let tests = [
            ("bengali", "bengali"),
            ("\"simple\" english", r#""\"simple\" english""#),
            ("", "\"\""),
            ("\"", "\"\\\"\""),
            ("\\", "\"\\\\\""),
        ];

        for (from, expected) in tests.iter() {
            let cs = Charset::try_from(*from).unwrap();
            println!("{:?}", cs);

            let mut out = Vec::new();
            cs.encode(&mut out).unwrap();
            assert_eq!(from_utf8(&out).unwrap(), *expected);
        }

        assert!(Charset::try_from("\r").is_err());
        assert!(Charset::try_from("\n").is_err());
        assert!(Charset::try_from("¹").is_err());
        assert!(Charset::try_from("²").is_err());
        assert!(Charset::try_from("\x00").is_err());
    }
}