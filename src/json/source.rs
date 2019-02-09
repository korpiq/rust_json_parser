use std::io::{stdin, Read};
use circular::Buffer;

//type Parseable = [u8];
pub type Parseable = StreamSource;
//JsonInput<'static>;

pub struct StreamSource {
}

/*

impl Parseable {
    pub fn new(data : &[u8]) -> Parseable {
        JsonInput {
            data
        }
    }
}

impl nom::AtEof for Parseable {
    fn at_eof (self : &Self) -> bool { false }
}

impl nom::InputIter for Parseable {
    type Item = u8;
    type RawItem = u8;
    type Iter = Iterator<Item = (usize, Self::Item)>;
    type IterElem = Iterator<Item = Self::Item>;

    fn iter_indices (self: &Self) -> <Self as nom::InputIter>::Iter { *self }

    fn iter_elements (self: &Self) -> <Self as nom::InputIter>::IterElem { *self }

    fn position<P> (self: &Self, p: P) -> std::option::Option<usize> { Option::None }

    fn slice_index (self: &Self, sz: usize) -> std::option::Option<usize> { Option::None }
}

impl nom::InputLength for Parseable {
    fn input_len (self: &Self) -> usize { 0 }
}

impl nom::InputTake for Parseable {
    fn take (self: &Self, sz: usize) -> Self { *self }

    fn take_split (self: &Self, sz: usize) -> (Self, Self) { (*self, *self) }
}

impl nom::Offset for Parseable {
    fn offset (self: &Self, self2: &Self) -> usize { 0 }
}

impl nom::ParseTo<f64> for Parseable {
    fn parse_to (self: &Self) -> std::option::Option<f64> { Option::None }

}

impl nom::Slice<std::ops::Range<usize>> for Parseable {
    fn slice (self: &Self, r: std::ops::Range<usize>) -> Self { *self }
}

impl nom::Slice<std::ops::RangeFrom<usize>> for Parseable {
    fn slice (self: &Self, r: std::ops::RangeFrom<usize>) -> Self { *self }
}

impl nom::Slice<std::ops::RangeTo<usize>> for Parseable {
    fn slice (self: &Self, r: std::ops::RangeTo<usize>) -> Self { *self }
}
*/
