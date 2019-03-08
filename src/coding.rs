//! This module contains the data structures and functions related
//! to actually encoding data with Huffman coding
use std::collections::HashMap;
use std::io;
use crate::queue::PriorityQueue;


#[inline]
fn mask(size: usize) -> u64 {
    if size == 64 {
        u64::max_value()
    } else {
        (1 << size) - 1
    }
}


// Like write_u64, but we may not write all the bytes
// if the trim size is low enough
#[inline]
fn write_u64_trimmed<W: io::Write>(writer: &mut W, mut num: u64, significant: usize) -> io::Result<()> {
    if significant == 0 {
        return Ok(())
    }
    let num_bytes = 1 + significant / 8;
    let mut bytes = [0; 8];
    for byte in bytes[..num_bytes].iter_mut() {
        *byte = num as u8;
        num >>= 8;
    }
    writer.write_all(&bytes[..num_bytes])
}

// uses reverse network order, because we write bits in from LSB to MSB
// in the u64, so we want the first byte to be the least significant
fn write_u64<W: io::Write>(writer: &mut W, num: u64) -> io::Result<()> {
    write_u64_trimmed(writer, num, 64)
}


/// A struct holding the frequencies of each character,
/// allowing us to estimate the probability of each character
pub struct Frequencies {
    // We simply don't store the pairs we don't need,
    // the other ones simply don't occurr in the file
    pairs: Vec<(u8, u8)>
}

impl Frequencies {
    /// Count the number of occurrences of each byte in order to build
    /// up a struct of Frequencies
    pub fn count_bytes<E, I : IntoIterator<Item=Result<u8, E>>>(bytes: I) -> Result<Self, E> {
        let mut acc: HashMap<u8, u64> = HashMap::new();
        for maybe_byte in bytes {
            let b = maybe_byte?;
            acc.insert(b, acc.get(&b).unwrap_or(&0) + 1);
        }
        // There will always be at least one byte
        let max = acc.values().max().unwrap();
        let mut pairs = Vec::with_capacity(acc.len());
        // This guarantees a consistent ordering of pairs, and thus of the H Tree
        for byte in 0..=255 {
            if let Some(v) = acc.get(&byte) {
                pairs.push(((v * 255 / max) as u8, byte))
            }
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
            ((num_buf[3] as usize) << 24) | 
            ((num_buf[2] as usize) << 16) |
            ((num_buf[1] as usize) << 8)  |
            (num_buf[0] as usize);
        let mut pair_buf = vec![0; num * 2];
        reader.read_exact(&mut pair_buf)?;
        let mut pairs = Vec::with_capacity(num);
        for i in 0..(pair_buf.len() - 1) {
            pairs.push((pair_buf[i + 1], pair_buf[i]));
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
    map: HashMap<u8, (u64, usize)>,
    eof: (u64, usize),
    shift: usize,
    scratch: u64
}

impl HuffWriter {
    pub fn from_tree(start_tree: HuffTree) -> Self {
        let mut trees = Vec::new();
        trees.push((start_tree, 0, 0));
        let mut map = HashMap::new();
        let mut eof = (0, 0);
        while let Some((tree, bits, shift)) = trees.pop() {
            match tree {
                HuffTree::Branch(left, right) => {
                    trees.push((*left, bits, shift + 1));
                    trees.push((*right, (1 << shift) | bits, shift + 1));
                }
                HuffTree::EOF => eof = (bits, shift),
                HuffTree::Known(byte) => { map.insert(byte, (bits, shift)); }
            }
        }
        HuffWriter { map, eof, shift: 0, scratch: 0 }
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
        // Since we used the same file to generate the map that we're reading from
        // The other branch should never be reached in practice
        if let Some(&(bits, bit_size)) = self.map.get(&byte) {
            self.write_bits(bits, bit_size, writer)
        } else {
            Ok(())
        }
    }

    /// Write the end of the transmission, flushing out the remaining bits, and writing
    /// the EOF symbol
    pub fn end_transmission<W: io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
        let (bits, bit_size) = self.eof;
        self.write_bits(bits, bit_size, writer)?;
        // this won't write anything if self.shift is 0, avoiding writing the last bytes twice
        write_u64_trimmed(writer, self.scratch, self.shift)
    }
}


mod test {
    use super::*;

    #[test]
    fn huff_tree_freqs_works() {
        let mut freqs = Frequencies { pairs: Vec::new() };
        freqs.pairs.push((70, 1));
        freqs.pairs.push((71, 2));
        freqs.pairs.push((69, 100));
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