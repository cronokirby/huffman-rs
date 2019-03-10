use std::fs::File;
use std::io;
use std::io::{Read, Seek};
use crate::structopt::StructOpt;
use crate::coding;


#[derive(Debug, StructOpt)]
#[structopt(name = "huffman")]
pub enum Opt {
    #[structopt(name = "encode")]
    /// Encode a file
    Encode {
        /// The input file to encode
        input: String,
        #[structopt(short = "o")]
        /// The output file to put the decoded text into
        output: String
    },
    #[structopt(name = "decode")]
    /// Decode a file
    Decode {
        /// The input file to decode
        input: String,
        #[structopt(short = "o")]
        /// The output file to put the decoded text into
        output: String
    }
}

impl Opt {
    /// Handle all the cases of the options, and run the corresponding
    /// sub programs.
    pub fn dispatch(self) -> io::Result<()> {
        match self {
            Opt::Decode { input, output } => decode(input, output),
            Opt::Encode { input, output } => encode(input, output)
        }
    }
}

fn encode(input: String, output: String) -> io::Result<()> {
    let mut input_file = File::open(input)?;
    let output_file = File::create(output)?;
    let mut output_writer = io::BufWriter::new(output_file);

    let input_copy = input_file.try_clone()?;
    let freqs = coding::Frequencies::count_bytes(input_copy.bytes())?;
    freqs.write(&mut output_writer)?;

    let tree = coding::HuffTree::from_freqs(&freqs);
    let mut encoder = coding::HuffWriter::from_tree(&tree);
    input_file.seek(io::SeekFrom::Start(0))?;
    for maybe_byte in input_file.bytes() {
        let byte = maybe_byte?;
        encoder.write_byte(byte, &mut output_writer)?;
    }
    encoder.end_transmission(&mut output_writer)
}

fn decode(input: String, output: String) -> io::Result<()> {
    let mut input_file = File::open(input)?;
    let output_file = File::create(output)?;
    let mut output_writer = io::BufWriter::new(output_file);

    let freqs = coding::Frequencies::read(&mut input_file)?;
    let tree = coding::HuffTree::from_freqs(&freqs);
    let mut reader = coding::HuffReader::new(&tree);

    for maybe_byte in input_file.bytes() {
        let byte = maybe_byte?;
        let can_feed = reader.feed(byte, &mut output_writer)?;
        if !can_feed {
            break;
        }
    }
    Ok(())
}
