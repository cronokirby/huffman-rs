use std::io;

#[macro_use]
extern crate structopt;
use structopt::StructOpt;

mod cli;
pub mod coding;
mod queue;


fn main() -> io::Result<()> {
    let opt = cli::Opt::from_args();
    opt.dispatch()
}
