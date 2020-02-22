use slotmap::{new_key_type, DenseSlotMap};
use std::collections::HashSet;

use crate::gene::{Gene, GeneKey};
use crate::lookup;

new_key_type! {pub struct CellKey; }

pub struct Cell {
    gene_lookup: lookup::Lookup<GeneKey>,
    genes: HashSet<GeneKey>,
}

impl Cell {
    pub fn new() -> Cell {
        Cell {
            gene_lookup: lookup::Lookup::new(),
            genes: HashSet::new(),
        }
    }

    pub fn add_gene2(&mut self, gene_key: GeneKey, coordinates: u32) {
        self.gene_lookup.add(coordinates, gene_key).unwrap();
        self.genes.insert(gene_key);
    }

    pub fn lookup_gene_id(&self, genes: &DenseSlotMap<GeneKey, Gene>, coordinates: u32) -> u32 {
        genes[*self.gene_lookup.find(coordinates)].id
    }

    pub fn has_gene(&self, gene_key: GeneKey) -> bool {
        self.genes.contains(&gene_key)
    }
}
