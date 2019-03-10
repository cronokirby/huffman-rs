use std::io;

extern crate structopt;
use structopt::StructOpt;

extern crate huffman;
use huffman::cli;


fn main() -> io::Result<()> {
    let opt = cli::Opt::from_args();
    opt.dispatch()
}
