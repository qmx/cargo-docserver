#[macro_use]
extern crate structopt;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "cargo")]
enum Cargo {
    #[structopt(name = "docserver")]
    Docserver {
        #[structopt(short = "p", long = "port", default_value = "4000")]
        port: u32,
    },
}
fn main() {
    let cfg = Cargo::from_args();
    println!("Hello, world! {:?}", cfg);
}
