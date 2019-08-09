use serde::{
    Deserialize,
    de::Deserializer,
    Serialize,
    ser::Serializer,
};

use crate::identifier::TypeIdentifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Optional<T: TypeIdentifier>(Option<Option<T>>);

impl<T: Serialize + TypeIdentifier> Serialize for Optional<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        <Option<Option<T>>>::serialize(&self.0, serializer)
    }
}


impl<'de, T: Deserialize<'de> + TypeIdentifier> Deserialize<'de> for Optional<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let inner = <Option<Option<T>>>::deserialize(deserializer)?;

        Ok(Optional(inner))
    }
}

impl<T: TypeIdentifier> From<Option<T>> for Optional<T> {
    fn from(option: Option<T>) -> Self {
        let option = if option.is_none() {
            None
        } else {
            Some(option)
        };

        Optional(option)
    }
}
