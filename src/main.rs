#[macro_use]
extern crate enum_map;
extern crate rand;
extern crate rand_pcg;

// XXX use pub mod to shut up unused warnings
pub mod gene;
pub mod lookup;
pub mod processor;
pub mod reaction;
pub mod stack;
pub mod triplet;
pub mod cell;

fn main() {
    println!("Hello, world!");
}
