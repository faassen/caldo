#[macro_use]
extern crate enum_map;

// XXX use pub mod to shut up unused warnings
pub mod gene;
pub mod lookup;
pub mod processor;
pub mod reaction;
pub mod stack;
pub mod triplet;

fn main() {
    println!("Hello, world!");
}
