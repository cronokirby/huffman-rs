use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Read;

extern crate argparse;
use argparse::{ArgumentParser, Store};


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

fn main() -> io::Result<()> {
    let mut corpus_file = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut corpus_file)
            .add_argument(&"CorpusFile", Store, "The path of the file we'll count frequencies from")
            .required();
        ap.parse_args_or_exit();
    }    
    let file = File::open(corpus_file)?;
    let map = build_byte_freqs(file.bytes())?;
    println!("Frequencies: {:?}", map);
    Ok(())
}
