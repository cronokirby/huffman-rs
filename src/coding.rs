//! This module contains the data structures and functions related
//! to actually encoding data with Huffman coding
use std::collections::HashMap;
use std::io;
use crate::queue::PriorityQueue;


/// Construct a map from byte to number occurrences, by counting them as they
/// come along through the Iterator. If the iterator fails at any point, this function
/// immediately returns the error.
/// This takes an iterator over an error, mainly to work nicely with the file api.
pub fn build_byte_freqs<E, I : IntoIterator<Item=Result<u8, E>>>(bytes: I) -> Result<HashMap<u8, i32>, E> {
    let mut acc = HashMap::new();
    for maybe_byte in bytes {
        let b = maybe_byte?;
        acc.insert(b, acc.get(&b).unwrap_or(&0) + 1);
    }
    Ok(acc)
}


/// Represents a Huffman decoding tree.
/// 
/// This structure is constructed using the probabilities or frequencies
/// for each of the symbols we want to encode: in our case, bytes.
/// Given this tree, we can easily decode a stream of bits as they arrive
/// by using them to navigate the tree until we arrive at a terminal node.
#[derive(Debug, PartialEq)]
pub enum HuffTree {
    /// Branch out into 2 subtrees
    Branch(Box<HuffTree>, Box<HuffTree>),
    /// We've reached the end of the tree, and can return a byte
    Known(u8),
    /// We've reached a character we don't recognise.
    /// In decoding, this means that we need to use the following bytes
    /// to output the next byte
    Unknown
}

impl HuffTree {
    pub fn from_freqs(map: HashMap<u8, i32>) -> Self {
        let mut q = PriorityQueue::with_capacity(map.len());
        for (byte, count) in map {
            q.insert(count, HuffTree::Known(byte));
        }
        q.insert(0, HuffTree::Unknown);
        while let Some(((count1, tree1), (count2, tree2))) = q.remove_two() {
            let branch = HuffTree::Branch(Box::new(tree1), Box::new(tree2));
            q.insert(count1 + count2, branch);
        }
        // The q will always have one left
        q.remove().unwrap().1
    }
}

#[inline]
fn mask(size: usize) -> u64 {
    if size == 64 {
        u64::max_value()
    } else {
        (1 << size) - 1
    }
}

fn write_u64<W: io::Write>(writer: &mut W, mut num: u64) -> io::Result<()> {
    let mut bytes = [0; 8];
    for byte in bytes.iter_mut().rev() {
        *byte = num as u8;
        num >>= 8;
    }
    writer.write_all(&bytes)
}


/// A writer using a hufftree to write bytes to some source
pub struct HuffWriter {
    map: HashMap<u8, (u64, usize)>,
    default: (u64, usize),
    shift: usize,
    scratch: u64
}

impl HuffWriter {
    pub fn from_tree(start_tree: HuffTree) -> Self {
        let mut trees = Vec::new();
        trees.push((start_tree, 0, 0));
        let mut map = HashMap::new();
        let mut default = (0, 0);
        while let Some((tree, bits, shift)) = trees.pop() {
            match tree {
                HuffTree::Branch(left, right) => {
                    trees.push((*left, bits, shift + 1));
                    trees.push((*right, (1 << shift) | bits, shift + 1));
                }
                HuffTree::Unknown => default = (bits, shift),
                HuffTree::Known(byte) => { map.insert(byte, (bits, shift)); }
            }
        }
        HuffWriter { map, default, shift: 0, scratch: 0 }
    }

    fn write_bits<W: io::Write>(&mut self, bits: u64, bit_size: usize, writer: &mut W) -> io::Result<()> {
        let bit_size_left = 64 - self.shift;
        if self.shift == 64 {
            let scratch = self.scratch;
            self.scratch = bits;
            self.shift = bit_size;
            write_u64(writer, scratch)
        } else if bit_size > bit_size_left {
            let to_write = ((bits & mask(bit_size_left)) << self.shift) | self.scratch;
            self.scratch = bits >> bit_size_left;
            self.shift = bit_size - bit_size_left;
            write_u64(writer, to_write)
        } else {
            self.scratch = (bits << self.shift) | self.scratch;
            self.shift += bit_size;
            Ok(())
        }
    }

    pub fn write_byte<W: io::Write>(&mut self, byte: u8, writer: &mut W) -> io::Result<()> {
        if let Some(&(bits, bit_size)) = self.map.get(&byte) {
            self.write_bits(bits, bit_size, writer)
        } else {
            let (bits, bit_size) = self.default;
            self.write_bits(bits, bit_size, writer)?;
            writer.write_all(&[byte])
        }
    }
}


mod test {
    use super::*;

    #[test]
    fn huff_tree_freqs_works() {
        let mut map = HashMap::new();
        map.insert(70, 1);
        map.insert(71, 2);
        map.insert(69, 100);
        let tree = HuffTree::Branch(
            Box::new(HuffTree::Branch(
                Box::new(HuffTree::Branch(
                    Box::new(HuffTree::Unknown), 
                    Box::new(HuffTree::Known(70))
                )),
                Box::new(HuffTree::Known(71))
            )),
            Box::new(HuffTree::Known(69))
        );
        assert_eq!(HuffTree::from_freqs(map), tree);
    }
}