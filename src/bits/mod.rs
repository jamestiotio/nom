//! bit level parsers
//!

#[macro_use]
mod macros;

pub mod streaming;
pub mod complete;

use crate::error::{ErrorKind, ParseError};
use crate::internal::{Err, IResult, Needed};
use crate::lib::std::ops::{AddAssign, RangeFrom, Shl, Shr};
use crate::traits::{InputIter, Slice, ErrorConvert};

///blah 
pub struct BitsInput<I> {
  i: I,
  index: usize,
}

/*
impl<I> From<((I, usize), ErrorKind)> for (I, ErrorKind) {
   fn from(e: ((I, usize), ErrorKind)) -> (I, ErrorKind) {
     ((e.0).0, e.1)
   }
}
*/

impl<I> From<NomError<(I, usize)>> for NomError<I> {
   fn from(e: NomError<(I, usize)>) -> NomError<I> {
     NomError{
      i: e.i.0,
      e: e.e
     }
   }
}

/// blah
pub struct NomError<I> {
  i: I,
  e: ErrorKind,
}


/// Converts a byte-level input to a bit-level input, for consumption by a parser that uses bits.
///
/// Afterwards, the input is converted back to a byte-level parser, with any remaining bits thrown
/// away.
///
/// # Example
/// ```ignore
/// # #[macro_use] extern crate nom;
/// # use nom::IResult;
/// use nom::bits::bits;
/// use nom::bits::complete::take_bits;
///
/// fn take_4_bits(input: &[u8]) -> IResult<&[u8], u64> {
///   bits( take_bits(4usize) )(input)
/// }
///
/// let input = vec![0xAB, 0xCD, 0xEF, 0x12];
/// let sl    = &input[..];
///
/// assert_eq!(take_4_bits( sl ), Ok( (&sl[1..], 0xA) ));
/// ```
pub fn bits<I, O, E1: ParseError<(I, usize)>+ErrorConvert<E2>, E2: ParseError<I>, P>(parser: P) -> impl Fn(I) -> IResult<I, O, E2>
where
  I: InputIter<Item = u8> + Slice<RangeFrom<usize>>,
  P: Fn((I, usize)) -> IResult<(I, usize), O, E1>,
{
  move |input: I| match parser((input, 0)) {
    Ok(((rest, offset), res)) => {
      let byte_index = offset / 8 + if offset % 8 == 0 { 0 } else { 1 };
      Ok((rest.slice(byte_index..), res))
    }
    Err(Err::Incomplete(n)) => Err(Err::Incomplete(n.map(|u| u / 8 + 1))),
    Err(Err::Error(e)) => Err(Err::Error(e.convert())),
    Err(Err::Failure(e)) => Err(Err::Failure(e.convert())),
  }
}

#[doc(hidden)]
pub fn bitsc<I, O, E1: ParseError<(I, usize)>+ErrorConvert<E2>, E2: ParseError<I>, P>(input: I, parser: P) -> IResult<I, O, E2>
where
  I: InputIter<Item = u8> + Slice<RangeFrom<usize>>,
  P: Fn((I, usize)) -> IResult<(I, usize), O, E1>,
{
  bits(parser)(input)
}

/// Counterpart to bits, bytes transforms its bit stream input into a byte slice for the underlying
/// parser, allowing byte-slice parsers to work on bit streams.
///
/// A partial byte remaining in the input will be ignored and the given parser will start parsing
/// at the next full byte.
///
/// ```ignore
/// # #[macro_use] extern crate nom;
/// # use nom::IResult;
/// # use nom::combinator::rest;
/// # use nom::sequence::tuple;
/// use nom::bits::{bits, bytes, streaming::take_bits};
///
/// fn parse(input: &[u8]) -> IResult<&[u8], (u8, u8, &[u8])> {
///   bits(tuple((
///     take_bits(4usize),
///     take_bits(8usize),
///     bytes(rest)
///   )))(input)
/// }
///
/// let input = &[0xde, 0xad, 0xbe, 0xaf];
///
/// assert_eq!(parse( input ), Ok(( &[][..], (0xd, 0xea, &[0xbe, 0xaf][..]) )));
/// ```
pub fn bytes<I, O, E1: ParseError<I>+ErrorConvert<E2>, E2: ParseError<(I, usize)>, P>(parser: P) -> impl Fn((I, usize)) -> IResult<(I, usize), O, E2>
where
  I: InputIter<Item = u8> + Slice<RangeFrom<usize>>,
  P: Fn(I) -> IResult<I, O, E1>,
{
  move |(input, offset): (I, usize)| {
    let inner = if offset % 8 != 0 {
      input.slice((1 + offset / 8)..)
    } else {
      input.slice((offset / 8)..)
    };
    match parser(inner) {
      Ok((rest, res)) => Ok(((rest, 0), res)),
      Err(Err::Incomplete(n)) => Err(Err::Incomplete(n.map(|u| u * 8))),
      Err(Err::Error(e)) => Err(Err::Error(e.convert())),
      Err(Err::Failure(e)) => Err(Err::Failure(e.convert())),
    }
  }
}

#[doc(hidden)]
pub fn bytesc<I, O, E1: ParseError<I>+ErrorConvert<E2>, E2: ParseError<(I, usize)>, P>(input: (I, usize), parser: P) -> IResult<(I, usize), O, E2>
where
  I: InputIter<Item = u8> + Slice<RangeFrom<usize>>,
  P: Fn(I) -> IResult<I, O, E1>,
{
  bytes(parser)(input)
}

