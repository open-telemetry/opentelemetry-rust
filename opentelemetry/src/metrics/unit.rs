use std::borrow::Cow;

/// Units denote underlying data units tracked by `Meter`s.
#[derive(Clone, Default, Debug, PartialEq, Hash)]
pub struct Unit(Cow<'static, str>);

impl Unit {
    /// Create a new `Unit` from an `Into<String>`
    pub fn new<S>(value: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Unit(value.into())
    }

    /// View unit as &str
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<str> for Unit {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
