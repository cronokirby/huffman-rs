use std::collections::HashMap;

fn build_byte_freqs<I : IntoIterator<Item=u8>>(bytes: I) -> HashMap<u8, i32> {
    let mut acc = HashMap::new();
    for b in bytes {
        acc.insert(b, acc.get(&b).unwrap_or(&0) + 1);
    }
    acc
}

fn main() {
    println!("Hello, world!");
}
