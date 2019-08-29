use std::ops::{self, RangeInclusive};

use bit_vec::BitVec;
use num_traits::{Num, PrimInt, Unsigned};

pub(crate) fn encode_normally_small_whole_number<N: PrimInt + ops::AddAssign<N> + Unsigned>(n: N) -> BitVec {
    let boundary = N::from(63).unwrap();
    if n <= boundary {
        encode_constrained_whole_number(n, N::zero()..=boundary)
    } else {
        unimplemented!()
    }
}

pub(crate) fn encode_constrained_whole_number<N: PrimInt + ops::AddAssign<N>>(n: N, range: RangeInclusive<N>) -> BitVec
{
    assert!(range.contains(&n));
    // calculate the mininum number of bits required to encode the number.
    let width = {
        let max_difference = *range.end() - *range.start();
        let type_width = N::zero().count_zeros();
        type_width - max_difference.leading_zeros()
    };

    let mut buffer = BitVec::from_elem(width as usize, false);

    // We only encode the difference between the lower bound and the value
    // we're enocding.
    let mut bits = n - *range.start();
    // We always encode the number in big endian format.
    let mut index = buffer.len();
    while bits != N::zero() {
        index -= 1;
        buffer.set(index, bits & N::one() == N::one());
        bits = bits.unsigned_shr(1);
    }

    buffer
}

pub(crate) fn encode_semi_constrained_whole_number<N: PrimInt>(n: N, lb: N) -> BitVec {
    unimplemented!()
}

fn encode_non_negative_binary_integer<N: Num>(mut n: N, width: usize) {
    let mut buffer = BitVec::from_elem(width, false);

    // We always encode the number in big endian format.
    let mut index = buffer.len();
    while n != N::zero() {
        index -= 1;
        buffer.set(index, (n & N::one()) == N::one());
        n >>= 1;
    }
}

