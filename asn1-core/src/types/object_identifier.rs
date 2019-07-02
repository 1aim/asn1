use failure::{ensure, Fallible};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename="ASN.1#ObjectIdentifier")]
pub struct ObjectIdentifier<T: AsRef<[u128]>>(pub T);

impl<A: AsRef<[u128]>> ObjectIdentifier<A> {
    pub fn new(inner: A) -> Fallible<Self> {
        ensure!(
            inner.as_ref().len() >= 2,
            "ObjectIdentifier requires at least two components."
        );

        Ok(Self(inner))
    }
}

impl<T: AsRef<[u128]>> AsRef<[u128]> for ObjectIdentifier<T> {
    fn as_ref(&self) -> &[u128] {
        self.0.as_ref()
    }
}
