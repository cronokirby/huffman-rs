//! This module contains the data structures and functions related
//! to actually encoding data with Huffman coding
use std::collections::HashMap;
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
enum HuffTree {
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
    fn from_freqs(map: HashMap<u8, i32>) -> Self {
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