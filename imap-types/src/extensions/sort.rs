use std::fmt::{Display, Formatter};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "bounded-static")]
use bounded_static::ToStatic;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::core::Atom;

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SortAlgorithm<'a> {
    Display,
    Other(SortAlgorithmOther<'a>),
}

impl<'a> From<Atom<'a>> for SortAlgorithm<'a> {
    fn from(value: Atom<'a>) -> Self {
        match value.as_ref().to_lowercase().as_ref() {
            "display" => Self::Display,
            _ => Self::Other(SortAlgorithmOther(value)),
        }
    }
}

impl Display for SortAlgorithm<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SortAlgorithm::Display => f.write_str("DISPLAY"),
            SortAlgorithm::Other(other) => f.write_str(other.as_ref()),
        }
    }
}

#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SortAlgorithmOther<'a>(Atom<'a>);

impl AsRef<str> for SortAlgorithmOther<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SortCriterion {
    pub reverse: bool,
    pub key: SortKey,
}

#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "bounded-static", derive(ToStatic))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SortKey {
    Arrival,
    Cc,
    Date,
    From,
    Size,
    Subject,
    To,
    // RFC5957
    /// Note: Only use when server advertised `SORT=DISPLAY`.
    DisplayFrom,
    // RFC5957
    /// Note: Only use when server advertised `SORT=DISPLAY`.
    DisplayTo,
}

impl AsRef<str> for SortKey {
    fn as_ref(&self) -> &str {
        match self {
            SortKey::Arrival => "ARRIVAL",
            SortKey::Cc => "CC",
            SortKey::Date => "DATE",
            SortKey::From => "FROM",
            SortKey::Size => "SIZE",
            SortKey::Subject => "SUBJECT",
            SortKey::To => "TO",
            SortKey::DisplayFrom => "DISPLAYFROM",
            SortKey::DisplayTo => "DISPLAYTO",
        }
    }
}
