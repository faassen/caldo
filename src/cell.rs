use crate::gene::{Gene, GeneKey};
use crate::lookup;
use std::collections::HashMap;
extern crate rand_pcg;

use rand::Rng;
use slotmap::{new_key_type, DenseSlotMap};

new_key_type! {struct CellKey; }

struct World {
    cells: DenseSlotMap<CellKey, Cell>,
    genes: DenseSlotMap<GeneKey, Gene>,
}

pub struct Cell {
    genes: HashMap<u32, GeneKey>,
    gene_lookup: lookup::Lookup<GeneKey>,
    // processors: Vec<Processor<'a>>,
}

impl Cell {
    pub fn new() -> Cell {
        Cell {
            genes: HashMap::new(),
            gene_lookup: lookup::Lookup::new(),
            // processors: Vec::new(),
        }
    }

    pub fn add_gene<R: Rng>(
        &mut self,
        genes: &mut DenseSlotMap<GeneKey, Gene>,
        code: &[u32],
        rng: &mut R,
    ) -> GeneKey {
        let id = self.create_gene_id(rng);
        let gene = Gene::new(id, code);
        let coordinates = gene.coordinates();
        let gene_key = genes.insert(gene);
        self.genes.insert(id, gene_key);
        self.gene_lookup.add(coordinates, gene_key).unwrap();
        gene_key
    }

    fn create_gene_id<R: Rng>(&self, rng: &mut R) -> u32 {
        // XXX if we want to supporting moving genes,
        // we in fact need a globally unique gene id, not
        // a locally unique one
        loop {
            let id: u32 = rng.gen();
            if self.genes.contains_key(&id) {
                continue;
            }
            return id;
        }
    }

    pub fn lookup_gene_id(&self, genes: &DenseSlotMap<GeneKey, Gene>, coordinates: u32) -> u32 {
        genes[*self.gene_lookup.find(coordinates)].id
    }

    pub fn get_gene_key(&self, gene_id: u32) -> Option<GeneKey> {
        self.genes.get(&gene_id).map(|&gene_key| gene_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_gene_id() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();

        let mut cell = Cell::new();
        let mut rng =
            rand_pcg::Pcg32::from_seed([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let gene1_key = cell.add_gene(&mut genes, &[3, 4], &mut rng);
        let gene1_id = genes[gene1_key].id;

        let gene2_key = cell.add_gene(&mut genes, &[5, 3], &mut rng);
        let gene2_id = genes[gene2_key].id;

        let found_gene1_key = cell.get_gene_key(gene1_id).unwrap();
        assert_eq!(found_gene1_key, gene1_key);

        let found_gene2_key = cell.get_gene_key(gene2_id).unwrap();
        assert_eq!(found_gene2_key, gene2_key);

        let lookup_gene_id = cell.lookup_gene_id(&genes, 3);
        assert_eq!(gene1_id, lookup_gene_id);

        let lookup_gene_id = cell.lookup_gene_id(&genes, 5);
        assert_eq!(gene2_id, lookup_gene_id);
    }
}
