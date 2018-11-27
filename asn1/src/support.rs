use chrono::prelude::*;
use bigint::BigUint;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ObjectId(pub(crate) Vec<BigUint>);

impl AsRef<[BigUint]> for ObjectId {
	fn as_ref(&self) -> &[BigUint] {
		&self.0
	}
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Time {
	UTC(DateTime<Utc>),
	Generalized(DateTime<FixedOffset>),
}
