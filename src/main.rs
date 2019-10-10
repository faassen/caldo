#[macro_use]
extern crate enum_map;

// XXX use pub mod to shut up unused warnings
pub mod processor;
pub mod instruction_lookup;
pub mod reaction;
pub mod stack;
pub mod triplet;

fn main() {
    println!("Hello, world!");
}
