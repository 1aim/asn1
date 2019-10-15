use std::ops::{self, Bound};

use num_traits::{PrimInt, Unsigned};

use super::Buffer;

pub fn encode_integer<N, R>(n: N, range: R) -> Buffer
where
    N: PrimInt + ops::BitAnd<Output = N> + Copy + ops::ShrAssign<u32>,
    R: ops::RangeBounds<N>,
{
    match (range.start_bound(), range.end_bound()) {
        (Bound::Included(&start), Bound::Included(&end)) => {
            encode_constrained_whole_number(n, start..=end)
        }
        (Bound::Excluded(&start), Bound::Included(&end)) => {
            encode_constrained_whole_number(n, start + N::one()..=end)
        }
        (Bound::Included(&start), Bound::Excluded(&end)) => {
            encode_constrained_whole_number(n, start..=end - N::one())
        }
        (Bound::Included(&start), Bound::Unbounded) => {
            encode_semi_constrained_whole_number(n, start)
        }
        (Bound::Excluded(&start), Bound::Unbounded) => {
            encode_semi_constrained_whole_number(n, start + N::one())
        }
        _ => unimplemented!(),
    }
}

pub fn encode_length<R: ops::RangeBounds<usize>>(len: usize, range: R) -> Buffer {
    match range.end_bound() {
        Bound::Unbounded => encode_unconstrained_length(len),
        _ => unimplemented!(),
    }
}

pub fn encode_unconstrained_length(len: usize) -> Buffer {
    match len {
        0..=127 => encode_non_negative_binary_integer(len, 8),
        128..=15999 => {
            let mut buffer = encode_non_negative_binary_integer(len, 16);
            buffer.set(0, true);

            buffer
        }
        _ => unimplemented!(),
    }
}

pub fn encode_constrained_whole_number<N>(n: N, range: ops::RangeInclusive<N>) -> Buffer
where
    N: PrimInt + ops::BitAnd<Output = N> + Copy + ops::ShrAssign<u32>,
{
    assert!(range.contains(&n));
    // We only encode the difference between the lower bound and the value
    // we're enocding.
    let bits = n - *range.start();

    let max_difference = bit_width(*range.end() - *range.start());
    encode_non_negative_binary_integer(bits, max_difference)
}

pub(crate) fn _encode_normally_small_whole_number<N>(n: N) -> Buffer
where
    N: PrimInt + ops::BitAnd<Output = N> + Copy + ops::ShrAssign<u32> + Unsigned,
{
    let mut buffer = Buffer::from_elem(1, false);
    let boundary = N::from(63).unwrap();
    if n <= boundary {
        buffer.push_field_list(encode_constrained_whole_number(n, N::zero()..=boundary));
        buffer
    } else {
        unimplemented!()
    }
}

pub(crate) fn encode_semi_constrained_whole_number<N>(n: N, lb: N) -> Buffer
where
    N: PrimInt + ops::BitAnd<Output = N> + Copy + ops::ShrAssign<u32>,
{
    encode_non_negative_binary_integer(n - lb, bit_width(n - lb))
}

fn encode_non_negative_binary_integer<N>(mut n: N, width: usize) -> Buffer
where
    N: PrimInt + ops::BitAnd<Output = N> + Copy + ops::ShrAssign<u32>,
{
    // calculate the mininum number of bits required to encode the number.

    let mut buffer = Buffer::from_elem(width, false);

    // We always encode the number in big endian format.
    let mut index = buffer.len();
    while n != N::zero() {
        index -= 1;
        buffer.set(index, (n & N::one()) == N::one());
        n >>= 1;
    }

    buffer
}

fn bit_width<N: PrimInt>(n: N) -> usize {
    let type_width = N::zero().count_zeros();
    (type_width - n.leading_zeros()) as usize
}
