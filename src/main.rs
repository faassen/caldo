#[macro_use]
extern crate enum_map;

// XXX use pub mod to shut up unused warnings
pub mod gene;
mod instruction;
mod reaction;
mod triplet;
mod stack;
mod instruction_lookup;

fn main() {
    println!("Hello, world!");
}
