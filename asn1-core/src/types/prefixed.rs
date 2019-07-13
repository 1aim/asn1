/*
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use typenum::marker_traits::Unsigned;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Prefixed<C: ConstClass = Context, N: Unsigned, T> {
    #[serde(skip)]
    phantom: std::marker::PhantomData<Identifier<C, N>>,
    value: T
}

pub struct Identifier<C: ConstClass, N: Unsigned> {
    class: PhantomData<C>,
    tag: PhantomData<N>,
}

*/
