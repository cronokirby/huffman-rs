use std::io;
#[macro_use]
extern crate criterion;
use criterion::Criterion;
extern crate huffman;
use huffman::coding;


struct EmptyWriter;

impl io::Write for EmptyWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}


fn wrap_byte(b: u8) -> Result<u8, ()> {
    Ok(b)
}

fn build_freqs(bytes: &Vec<u8>) -> coding::Frequencies {
    let iter1 = bytes.iter().map(|b| wrap_byte(*b));
    coding::Frequencies::count_bytes(iter1).unwrap()
}

fn build_tree(freqs: &coding::Frequencies) -> coding::HuffTree {
    coding::HuffTree::from_freqs(freqs)
}

fn encode(bytes: &Vec<u8>, tree: &coding::HuffTree) {
    let mut encoder = coding::HuffWriter::from_tree(tree);
    let mut writer = EmptyWriter;
    for byte in bytes {
        encoder.write_byte(*byte, &mut writer).unwrap();
    }
    encoder.end_transmission(&mut writer).unwrap();
}

fn encoding_benchmark(c: &mut Criterion) {
    let mut bytes: Vec<u8> = Vec::with_capacity(256 * 1000);
    for _ in 0..1000 {
        for b in 0..=255 {
            bytes.push(b);
        }
    }
    let bytes1 = bytes.clone();
    let freqs = build_freqs(&bytes);
    let tree = build_tree(&freqs);
    c.bench_function("building freqs", move |b| b.iter(|| {
        build_freqs(&bytes);
    }));
    c.bench_function("building tree", move |b| b.iter(|| {
        build_tree(&freqs);
    }));
    c.bench_function("encoding with tree", move |b| b.iter(|| {
        encode(&bytes1, &tree);
    }));


    //c.bench_function("encoding bytes", move |b| b.iter(|| encode(&mut bytes)));
}

criterion_group!(benches, encoding_benchmark);
criterion_main!(benches);