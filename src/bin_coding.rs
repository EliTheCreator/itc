use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::event_tree::EventTree;
use crate::id_tree::IdTree;
use crate::stamp::Stamp;


#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    EndOfEncoding,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EndOfEncoding => write!(f, "Unexpected end of encoding encountered."),
        }
    }
}

impl Error for ParseError {}


#[derive(Debug)]
struct BitWriter {
    bytes: Vec<u8>,
    current: u8,
    bit_offset: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            current: 0,
            bit_offset: 8,
        }
    }

    pub fn write_bit(&mut self, bit: bool) {
        self.bit_offset -= 1;

        if bit {
            self.current |= 1 << self.bit_offset
        }

        if self.bit_offset == 0 {
            self.bit_offset = 8;
            self.bytes.push(self.current);
            self.current = 0;
        }
    }

    pub fn write_bits(&mut self, bits: u32, bit_count: u8) {
        for offset in (0..bit_count).rev() {
            let bit = (bits>>offset) & 1;
            self.write_bit(bit == 1);
        }
    }

    pub fn finalize(mut self) -> Box<[u8]> {
        if self.current != 0 {
            self.bytes.push(self.current);
        }

        self.bytes.into_boxed_slice()
    }
}

struct Encoder {
    bit_writer: BitWriter,
}

impl Encoder {
    pub fn new() -> Self {
        Self { bit_writer: BitWriter::new() }
    }

    pub fn encode_stamp(mut self, stamp: &Stamp) -> Box<[u8]> {
        self.bit_writer = BitWriter::new();
        self.encode_id_tree_impl(&stamp.i);
        self.encode_event_tree_impl(&stamp.e);
        self.bit_writer.finalize()
    }

    pub fn encode_id_tree(mut self, id_tree: &IdTree) -> Box<[u8]> {
        self.encode_id_tree_impl(id_tree);
        self.bit_writer.finalize()
    }

    pub fn encode_event_tree(mut self, event_tree: &EventTree) -> Box<[u8]> {
        self.encode_event_tree_impl(event_tree);
        self.bit_writer.finalize()
    }

    fn encode_id_tree_impl(&mut self, id_tree: &IdTree) {
        use crate::id_tree::IdTree::*;
        match id_tree {
            Leaf { i: false } => self.bit_writer.write_bits(0, 3),
            Leaf { i: true } => self.bit_writer.write_bits(1, 3),
            Node { left, right } => {
                match (left.as_ref(), right.as_ref()) {
                    (Leaf { i: false }, _) => {
                        self.bit_writer.write_bits(1, 2);
                        self.encode_id_tree_impl(right);
                    },
                    (_, Leaf { i: false }) => {
                        self.bit_writer.write_bits(2, 2);
                        self.encode_id_tree_impl(left);
                    },
                    (_, _) => {
                        self.bit_writer.write_bits(3, 2);
                        self.encode_id_tree_impl(left);
                        self.encode_id_tree_impl(right);
                    },
                }
            },
        }
    }

    fn encode_event_tree_impl(&mut self, event_tree: &EventTree) {
        use crate::event_tree::EventTree::*;
        match event_tree {
            Leaf { n } => {
                self.encode_n(*n);
            },
            Node { n: 0, left, right } => {
                self.bit_writer.write_bit(false);
                match (left.as_ref(), right.as_ref()) {
                    (Leaf { n: 0 }, _) => {
                        self.bit_writer.write_bits(0, 2);
                        self.encode_event_tree_impl(right);
                    },
                    (_, Leaf { n: 0 }) => {
                        self.bit_writer.write_bits(1, 2);
                        self.encode_event_tree_impl(left);
                    },
                    (_, _) => {
                        self.bit_writer.write_bits(2, 2);
                        self.encode_event_tree_impl(left);
                        self.encode_event_tree_impl(right);
                    },
                }
            },
            Node { n, left, right } => {
                self.bit_writer.write_bit(false);
                match (left.as_ref(), right.as_ref()) {
                    (Leaf { n: 0 }, _) => {
                        self.bit_writer.write_bits(3, 2);
                        self.bit_writer.write_bits(0, 2);
                        self.encode_n(*n);
                        self.encode_event_tree_impl(right);
                    },
                    (_, Leaf { n: 0 }) => {
                        self.bit_writer.write_bits(3, 2);
                        self.bit_writer.write_bits(1, 2);
                        self.encode_n(*n);
                        self.encode_event_tree_impl(left);
                    },
                    (_, _) => {
                        self.bit_writer.write_bits(3, 2);
                        self.bit_writer.write_bit(true);
                        self.encode_n(*n);
                        self.encode_event_tree_impl(left);
                        self.encode_event_tree_impl(right);
                    },
                }
            }
        }
    }

    fn encode_n(&mut self, n: u32) {
        self.bit_writer.write_bit(true);
        self.encode_u32(n, 2);
    }

    fn encode_u32(&mut self, n: u32, b: u8) {
        let two_to_pow_b = 2<<b;
        if n < two_to_pow_b {
            self.bit_writer.write_bit(false);
            self.bit_writer.write_bits(n, b);
        } else {
            self.bit_writer.write_bit(true);
            self.encode_u32(n-two_to_pow_b, b+1);
        }
    }
}


impl Into<Box<[u8]>> for &Stamp {
    fn into(self) -> Box<[u8]> {
        Encoder::new().encode_stamp(self)
    }
}

impl Into<Box<[u8]>> for Stamp {
    fn into(self) -> Box<[u8]> {
        (&self).into()
    }
}

impl Into<Box<[u8]>> for &IdTree {
    fn into(self) -> Box<[u8]> {
        Encoder::new().encode_id_tree(self)
    }
}

impl Into<Box<[u8]>> for IdTree {
    fn into(self) -> Box<[u8]> {
        (&self).into()
    }
}

impl Into<Box<[u8]>> for &EventTree {
    fn into(self) -> Box<[u8]> {
        Encoder::new().encode_event_tree(self)
    }
}

impl Into<Box<[u8]>> for EventTree {
    fn into(self) -> Box<[u8]> {
        (&self).into()
    }
}


#[derive(Debug)]
struct BitIterator<'a> {
    byte_offset: usize,
    bit_offset: u8,
    bits: &'a [u8],
}

impl<'a> BitIterator<'a> {
    pub fn new(bits: &'a [u8]) -> Self {
        BitIterator {
            byte_offset: 0,
            bit_offset: 7,
            bits: bits,
        }
    }
}

impl<'a> Iterator for BitIterator<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.byte_offset < self.bits.len() {
            let value = self.bits[self.byte_offset] >> self.bit_offset & 0x1;
            if self.bit_offset == 0 {
                self.bit_offset = 7;
                self.byte_offset += 1;
            } else {
                self.bit_offset -= 1;
            }
            Some(value == 1)
        } else {
            None
        }
    }
}

struct Parser<'a> {
    bit_iter: BitIterator<'a>,
}

impl<'a> Parser<'a> {
    fn new(bits: &'a[u8]) -> Self {
        Self {
            bit_iter: BitIterator::new(bits)
        }
    }

    fn parse_stamp(mut self) -> Option<Stamp> {
        let id_tree = self.parse_id_tree()?;
        let event_tree = self.parse_event_tree()?;

        Some(Stamp::new(id_tree, event_tree))
    }

    fn parse_id_tree(&mut self) -> Option<IdTree> {
        match (self.next(), self.next()) {
            (Some(false), Some(false)) => {
                self.next()
                    .map(|bit| IdTree::leaf(bit))
            },
            (Some(false), Some(true)) => {
                self.parse_id_tree()
                    .map(|right| IdTree::node(
                        Box::new(IdTree::zero()),
                        Box::new(right),
                    ))
            },
            (Some(true), Some(false)) => {
                self.parse_id_tree()
                    .map(|left| IdTree::node(
                        Box::new(left),
                        Box::new(IdTree::zero()),
                    ))
            },
            (Some(true), Some(true)) => {
                let left = self.parse_id_tree()?;
                let right = self.parse_id_tree()?;

                Some(IdTree::node(
                    Box::new(left),
                    Box::new(right),
                ))
            },
            _ => None,
        }
    }

    fn parse_event_tree(&mut self) -> Option<EventTree> {
        match self.next() {
            Some(false) => self.parse_event_tree_node(),
            Some(true) => self.parse_event_tree_leaf(),
            None => None,
        }
    }

    fn parse_event_tree_node(&mut self) -> Option<EventTree> {
        match (self.next(), self.next()) {
            (Some(false), Some(false)) => {
                self.parse_event_tree()
                    .map(|right| EventTree::node(
                        0,
                        Box::new(EventTree::zero()),
                        Box::new(right),
                    ))
            },
            (Some(false), Some(true)) => {
                self.parse_event_tree()
                    .map(|left| EventTree::node(
                        0,
                        Box::new(left),
                        Box::new(EventTree::zero()),
                    ))
            },
            (Some(true), Some(false)) => {
                let left = self.parse_event_tree()?;
                let right = self.parse_event_tree()?;

                Some(EventTree::node(
                    0,
                    Box::new(left),
                    Box::new(right),
                ))
            },
            (Some(true), Some(true)) => {
                match self.next() {
                    Some(false) => {
                        let condition = self.next()?;
                        let n = self.parse_n()?;
                        let mut left = EventTree::zero();
                        let mut right = self.parse_event_tree()?;

                        if condition {
                            (left, right) = (right, left)
                        }

                        Some(EventTree::node(
                            n,
                            Box::new(left),
                            Box::new(right)
                        ))
                    }
                    Some(true) => {
                        let n = self.parse_n()?;
                        let left = self.parse_event_tree_node()?;
                        let right = self.parse_event_tree_node()?;

                        Some(EventTree::node(
                            n,
                            Box::new(left),
                            Box::new(right)
                        ))
                    },
                    None => None,
                }
            },
            _ => None,
        }
    }

    fn parse_event_tree_leaf(&mut self) -> Option<EventTree> {
        let n = self.parse_u32(2)?;
        Some(EventTree::leaf(n))
    }

    fn parse_n(&mut self) -> Option<u32> {
        if self.next()? {
            self.parse_u32(2)
        } else {
            None
        }
    }

    fn parse_u32(&mut self, b: u32) -> Option<u32> {
        let condition = self.next()?;
        if condition {
            Some((1<<b) + self.parse_u32(b+1)?)
        } else {
            let mut n: u32 = 0;
            for offset in (0..b).rev() {
                let bit = self.next()? as u32;
                n += bit<<offset;
            }
            Some(n)
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        self.bit_iter.next()
    }
}


impl TryFrom<&[u8]> for Stamp {
    type Error = ParseError;

    fn try_from(bits: &[u8]) -> std::result::Result<Self, Self::Error> {
        Parser::new(bits).parse_stamp().ok_or(ParseError::EndOfEncoding)
    }
}

impl TryFrom<&[u8]> for IdTree {
    type Error = ParseError;

    fn try_from(bits: &[u8]) -> std::result::Result<Self, Self::Error> {
        Parser::new(bits).parse_id_tree().ok_or(ParseError::EndOfEncoding)
    }
}

impl TryFrom<&[u8]> for EventTree {
    type Error = ParseError;

    fn try_from(bits: &[u8]) -> std::result::Result<Self, Self::Error> {
        Parser::new(bits).parse_event_tree().ok_or(ParseError::EndOfEncoding)
    }
}


#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::event_tree::EventTree;
    use crate::id_tree::IdTree;
    use crate::stamp::Stamp;


    #[test]
    fn bin_encode() {
        let stamp = Stamp::new(
            IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero())),
            EventTree::node(2, Box::new(EventTree::leaf(2)), Box::new(EventTree::leaf(0))),
        );

        let b: Box<[u8]> = stamp.into();
        let control: Box<[u8]> = vec![0b1000_1011,0b0110_1010,0b1000_0000].into_boxed_slice();

        assert_eq!(b, control);
    }

    #[test]
    fn bin_decode() {
        let bits: Box<[u8]> = vec![0b1000_1011,0b0110_1011,0b1101_1101].into_boxed_slice();

        let stamp = Stamp::try_from(bits).unwrap();
        let control = Stamp::new(
            IdTree::node(Box::new(IdTree::one()), Box::new(IdTree::zero())),
            EventTree::node(2, Box::new(EventTree::leaf(57)), Box::new(EventTree::leaf(0))),
        );

        assert_eq!(stamp, control)
    }
}
