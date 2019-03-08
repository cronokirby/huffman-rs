use std::fs::File;
use std::io;
use std::io::Read;
use crate::structopt::StructOpt;
use crate::coding;

#[derive(Debug, StructOpt)]
#[structopt(name = "huffman")]
pub enum Opt {
    #[structopt(name = "encode")]
    /// Encode a file
    Encode {
        /// The path of the corpus to use
        /// 
        /// Character frequency will be calculated from this file
        /// by counting the occurrences of each character.
        corpus: String,
        /// The input file to encode
        input: String,
        #[structopt(short = "o")]
        /// The output file to put the decoded text into
        output: Option<String>
    },
    #[structopt(name = "decode")]
    /// Decode a file
    Decode {
        /// The path of the corpus to use
        /// 
        /// This must match the corpus used to encode the file.
        corpus: String,
        /// The input file to decode
        input: String,
        #[structopt(short = "o")]
        /// The output file to put the decoded text into
        output: Option<String>
    }
}

impl Opt {
    /// Handle all the cases of the options, and run the corresponding
    /// sub programs.
    pub fn dispatch(self) -> io::Result<()> {
        match self {
            Opt::Decode { .. } => unimplemented!(),
            Opt::Encode { corpus, input, output } => encode(corpus, input, output)
        }
    }
}

fn encode(corpus: String, input: String, output: Option<String>) -> io::Result<()> {
    let real_output = output.unwrap_or_else(|| {
        let mut real = input.clone();
        real.push_str(".out");
        real
    });
    let corpus_file = File::open(corpus)?;
    let input_file = File::open(input)?;
    let output_file = File::create(real_output)?;
    let mut output_writer = io::BufWriter::new(output_file);

    let freqs = coding::build_byte_freqs(corpus_file.bytes())?;
    let tree = coding::HuffTree::from_freqs(freqs);
    let mut encoder = coding::HuffWriter::from_tree(tree);
    for maybe_byte in input_file.bytes() {
        let byte = maybe_byte?;
        encoder.write_byte(byte, &mut output_writer)?;
    }
    Ok(())
}
