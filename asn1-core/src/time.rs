use chrono::prelude::*;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Time {
    UTC(DateTime<Utc>),
    Generalized(DateTime<FixedOffset>),
}
