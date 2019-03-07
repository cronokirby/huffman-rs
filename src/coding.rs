use std::collections::HashMap;

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

/// Represents a Huffman decoding tree
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

