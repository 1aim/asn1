use std::ops::Deref;

use crate::Tag;

#[derive(Default, Debug)]
/// Wrapper for a value to be encoded or decoded.
pub struct Value<V> {
    inner: V,

    pub implicit: Option<Tag>,
    pub explicit: Option<Tag>,
}

impl<V> Value<V> {
    /// Create a new value.
    pub fn new(value: V) -> Value<V> {
        Value {
            inner: value,

            implicit: None,
            explicit: None,
        }
    }

    pub fn as_ref(&self) -> Value<&V> {
        Value {
            inner: &self.inner,

            implicit: self.implicit,
            explicit: self.explicit,
        }
    }

    pub fn map<F, O>(self, f: F) -> Value<O>
    where
        F: FnOnce(V) -> O,
    {
        Value {
            inner: f(self.inner),

            implicit: self.implicit,
            explicit: self.explicit,
        }
    }

    /// Create a new `Value` with the same metadata as `self` and the passed
    /// `value`.
    pub fn apply<O>(&self, value: O) -> Value<O> {
        Value {
            inner: value,

            implicit: self.implicit,
            explicit: self.explicit,
        }
    }

    /// Take ownership of the wrapped value.
    pub fn into_inner(self) -> V {
        self.inner
    }

    /// Set the implicit tag for this value.
    pub fn implicit(mut self, tag: Tag) -> Self {
        self.implicit = Some(tag);
        self
    }

    /// Set the explicit tag of this value.
    pub fn explicit(mut self, tag: Tag) -> Self {
        self.explicit = Some(tag);
        self
    }
}

impl<V> From<V> for Value<V> {
    fn from(value: V) -> Value<V> {
        Value::new(value)
    }
}

impl<V> Deref for Value<V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
