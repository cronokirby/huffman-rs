#[macro_use]
extern crate structopt;
use structopt::StructOpt;

mod cli;
pub mod coding;
mod queue;


fn main() {
    let opt = cli::Opt::from_args();
    println!("{:?}", opt);
}
