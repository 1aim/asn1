use derefable::Derefable;

#[derive(Clone, Debug, PartialEq)]
pub struct OctetString<A: AsRef<[u8]>> {
    inner: A,
}

impl<A: AsRef<[u8]>> OctetString<A> {
    pub fn new(inner: A) -> Self {
        Self { inner }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.inner.as_ref().to_owned()
    }
}
