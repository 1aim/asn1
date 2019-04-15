#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ObjectIdentifier<T: AsRef<[u8]>>(pub T);

impl<T: AsRef<[u8]>> AsRef<[u8]> for ObjectIdentifier<T> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
