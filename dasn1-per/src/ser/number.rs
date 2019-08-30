use std::ops::{self, Bound};

use bit_vec::BitVec;
use num_traits::{PrimInt, Unsigned};

use super::Buffer;

pub fn encode_integer<N, R>(n: N, range: R) -> Buffer
    where N: PrimInt + ops::BitAnd<Output=N> + Copy + ops::ShrAssign<u32> + Unsigned,
          R: ops::RangeBounds<N>
{
    match (range.start_bound(), range.end_bound()) {
        (Bound::Included(&start), Bound::Included(&end)) => {
            encode_constrained_whole_number(n, start..=end)
        }
        (Bound::Excluded(&start), Bound::Included(&end)) => {
            encode_constrained_whole_number(n, start+N::one()..=end)
        }
        (Bound::Included(&start), Bound::Excluded(&end)) => {
            encode_constrained_whole_number(n, start..=end-N::one())
        }
        _ => unimplemented!()
    }
}

pub fn encode_constrained_whole_number<N>(n: N, range: ops::RangeInclusive<N>)
    -> Buffer
    where N: PrimInt + ops::BitAnd<Output=N> + Copy + ops::ShrAssign<u32> + Unsigned,
{
    assert!(range.contains(&n));
    // calculate the mininum number of bits required to encode the number.
    let width = {
        let max_difference = *range.end() - *range.start();
        let type_width = N::zero().count_zeros();
        type_width - max_difference.leading_zeros()
    };

    // We only encode the difference between the lower bound and the value
    // we're enocding.
    let bits = n - *range.start();

    encode_non_negative_binary_integer(bits, width as usize)
}

pub(crate) fn encode_normally_small_whole_number<N>(n: N)
    -> Buffer
    where N: PrimInt + ops::BitAnd<Output=N> + Copy + ops::ShrAssign<u32> + Unsigned
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

pub(crate) fn encode_semi_constrained_whole_number<N: PrimInt>(n: N, lb: N) -> BitVec {
    unimplemented!()
}

fn encode_non_negative_binary_integer<N>(mut n: N, width: usize)
    -> Buffer
    where N: PrimInt + ops::BitAnd<Output=N> + Copy + ops::ShrAssign<u32> + Unsigned
{
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
