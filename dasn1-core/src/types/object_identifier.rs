use std::ops;

use failure::{ensure, Fallible};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename = "ASN.1#ObjectIdentifier")]
pub struct ObjectIdentifier(Vec<u32>);

impl ObjectIdentifier {
    pub fn new(inner: Vec<u32>) -> Fallible<Self> {
        ensure!(
            inner.len() >= 2,
            "ObjectIdentifier requires at least two components."
        );

        Ok(Self(inner))
    }
}

impl AsRef<[u32]> for ObjectIdentifier {
    fn as_ref(&self) -> &[u32] {
        self.0.as_ref()
    }
}

impl ops::Deref for ObjectIdentifier {
    type Target = Vec<u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for ObjectIdentifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
