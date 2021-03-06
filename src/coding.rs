//! This module contains the data structures and functions related
//! to actually encoding data with Huffman coding
use std::io;
use crate::queue::PriorityQueue;


// Like write_u64, but we may not write all the bytes
// if the trim size is low enough
#[inline]
fn write_u128_trimmed<W: io::Write>(writer: &mut W, mut num: u128, significant: usize) -> io::Result<()> {
    if significant == 0 {
        return Ok(())
    }
    let num_bytes = (significant - 1) / 8 + 1;
    let mut bytes = [0; 16];
    for byte in bytes[..num_bytes].iter_mut() {
        *byte = num as u8;
        num >>= 8;
    }
    writer.write_all(&bytes[..num_bytes])
}

// uses reverse network order, because we write bits in from LSB to MSB
// in the u64, so we want the first byte to be the least significant
fn write_u128<W: io::Write>(writer: &mut W, num: u128) -> io::Result<()> {
    write_u128_trimmed(writer, num, 128)
}


/// A struct holding the frequencies of each character,
/// allowing us to estimate the probability of each character
#[derive(Clone, Debug)]
pub struct Frequencies {
    // We simply don't store the pairs we don't need,
    // the other ones simply don't occurr in the file
    pairs: Vec<(u8, u8)>
}

impl Frequencies {
    /// Count the number of occurrences of each byte in order to build
    /// up a struct of Frequencies
    pub fn count_bytes<E, I : IntoIterator<Item=Result<u8, E>>>(bytes: I) -> Result<Self, E> {
        let mut acc: Vec<u64> = vec![0;256];
        for maybe_byte in bytes {
            let b = maybe_byte?;
            // Always fine since the byte is in the index
            acc[b as usize] += 1;
        }
        // There will always be at least one byte
        let max = acc.iter().max().unwrap();
        let mut pairs = Vec::with_capacity(acc.len());
        // This guarantees a consistent ordering of pairs, and thus of the H Tree
        let mut byte = 0;
        for &count in &acc {
            if count != 0 {
                pairs.push(((count * 255 / max) as u8, byte as u8));
            }
            byte += 1
        }
        // Sort pairs in reverse order by count
        pairs.sort_by(|(count1, _), (count2, _)| count2.cmp(count1));
        Ok(Frequencies { pairs })
    }

    /// This function writes the frequencies as a sequence of
    /// (byte, frequency) pairs, preceded by the number of pairs
    /// it can read.
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut len = self.pairs.len() as u32;
        let mut bytes = [0; 4];
        for byte in bytes.iter_mut().rev() {
            *byte = len as u8;
            len >>= 8;
        }
        writer.write_all(&bytes)?;
        for &(count, byte) in &self.pairs {
            writer.write_all(&[byte, count])?;
        }
        Ok(())
    }

    /// Attempt to read the frequencies from a some source
    pub fn read<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut num_buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut num_buf)?;
        let num = 
            ((num_buf[0] as usize) << 24) | 
            ((num_buf[1] as usize) << 16) |
            ((num_buf[2] as usize) << 8)  |
            (num_buf[3] as usize);
        let mut pair_buf = vec![0; num * 2];
        reader.read_exact(&mut pair_buf)?;
        let mut pairs = Vec::with_capacity(num);
        let mut i = 0;
        while i < pair_buf.len() - 1 {
            pairs.push((pair_buf[i + 1], pair_buf[i]));
            i += 2
        }
        Ok(Frequencies { pairs })
    }
}


/// Represents a Huffman decoding tree.
/// 
/// This structure is constructed using the probabilities or frequencies
/// for each of the symbols we want to encode: in our case, bytes.
/// Given this tree, we can easily decode a stream of bits as they arrive
/// by using them to navigate the tree until we arrive at a terminal node.
#[derive(Clone, Debug, PartialEq)]
pub enum HuffTree {
    /// Branch out into 2 subtrees
    Branch(Box<HuffTree>, Box<HuffTree>),
    /// We've reached the end of the tree, and can return a byte
    Known(u8),
    /// This is used to encode the end of the transmission
    EOF
}

impl HuffTree {
    pub fn from_freqs(freqs: &Frequencies) -> Self {
        let pairs: Vec<_> = freqs.pairs.iter().map(|&(count, byte)| {
            (count as u64, HuffTree::Known(byte))
        }).collect();
        let mut q = PriorityQueue::from_data(pairs);
        q.insert(0, HuffTree::EOF);
        while let Some(((count1, tree1), (count2, tree2))) = q.remove_two() {
            let branch = HuffTree::Branch(Box::new(tree1), Box::new(tree2));
            q.insert(count1 + count2, branch);
        }
        // The q will always have one left
        q.remove().unwrap().1
    }
}



/// A writer using a hufftree to write bytes to some source
pub struct HuffWriter {
    map: Box<[(u128, usize); 256]>,
    eof: (u128, usize),
    shift: usize,
    scratch: u128
}

impl HuffWriter {
    pub fn from_tree(start_tree: &HuffTree) -> Self {
        let mut trees = Vec::new();
        trees.push((start_tree, 0, 0));
        // Uninitialized values are never actually reached
        let mut map = Box::new([(0, 0); 256]);
        let mut eof = (0, 0);
        while let Some((tree, bits, shift)) = trees.pop() {
            match tree {
                HuffTree::Branch(left, right) => {
                    trees.push((left, bits, shift + 1));
                    trees.push((right, (1 << shift) | bits, shift + 1));
                }
                HuffTree::EOF => eof = (bits, shift),
                HuffTree::Known(byte) => { map[*byte as usize] = (bits, shift) }
            }
        }
        HuffWriter { map, eof, shift: 0, scratch: 0 }
    }

    fn write_bits<W: io::Write>(&mut self, bits: u128, bit_size: usize, writer: &mut W) -> io::Result<()> {
        self.scratch |= bits << self.shift; 
        self.shift += bit_size;
        if self.shift >= 128 {
            self.shift -= 128;
            let to_write = self.scratch;
            self.scratch = bits >> (bit_size - self.shift);
            write_u128(writer, to_write)
        } else {
            Ok(())
        }
    }

    pub fn write_byte<W: io::Write>(&mut self, byte: u8, writer: &mut W) -> io::Result<()> {
        let (bits, bit_size) = self.map[byte as usize];
        self.write_bits(bits, bit_size, writer)
    }

    /// Write the end of the transmission, flushing out the remaining bits, and writing
    /// the EOF symbol
    pub fn end_transmission<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        let (bits, bit_size) = self.eof;
        self.write_bits(bits, bit_size, writer)?;
        // this won't write anything if self.shift is 0, avoiding writing the last bytes twice
        write_u128_trimmed(writer, self.scratch, self.shift)
    }
}


/// A struct allowing us to incrementally feed in bits
/// (one byte at a time) and have it decode them using a
/// Huffman tree
pub struct HuffReader<'a> {
    top_tree: &'a HuffTree,
    tree: &'a HuffTree,
}

impl <'a> HuffReader<'a> {
    pub fn new(tree: &'a HuffTree) -> Self {
        HuffReader { top_tree: tree, tree }
    }

    /// Feed a byte to this reader
    /// Return true if the reader can continue to accept input
    pub fn feed<W: io::Write>(&mut self, mut byte: u8, writer: &mut W) -> io::Result<bool> {
        let mut i = 0;
        while i < 8 {
            match self.tree {
                HuffTree::Branch(left, right) => {
                    if byte & 1 == 0 {
                        self.tree = &left;
                    } else {
                        self.tree = &right;
                    }
                    byte >>= 1;
                    i += 1;
                }
                HuffTree::Known(byte) => {
                    writer.write_all(&[*byte])?;
                    self.tree = self.top_tree;
                }
                HuffTree::EOF => return Ok(false)
            }
        }
        Ok(true)
    }
}


#[cfg(test)]
mod test {
    use super::{HuffTree, Frequencies};

    #[test]
    fn huff_tree_freqs_works() {
        let mut freqs = Frequencies { pairs: Vec::new() };
        freqs.pairs.push((100, 69));
        freqs.pairs.push((2, 71));
        freqs.pairs.push((1, 70));
        let tree = HuffTree::Branch(
            Box::new(HuffTree::Branch(
                Box::new(HuffTree::Branch(
                    Box::new(HuffTree::EOF), 
                    Box::new(HuffTree::Known(70))
                )),
                Box::new(HuffTree::Known(71))
            )),
            Box::new(HuffTree::Known(69))
        );
        assert_eq!(HuffTree::from_freqs(&freqs), tree);
    }
}