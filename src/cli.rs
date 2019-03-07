use crate::structopt::StructOpt;

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