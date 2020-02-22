use slotmap::{new_key_type, DenseSlotMap};

use crate::gene::{Gene, GeneKey};
use crate::lookup;

new_key_type! {pub struct CellKey; }

pub struct Cell {
    gene_lookup: lookup::Lookup<GeneKey>,
}

impl Cell {
    pub fn new() -> Cell {
        Cell {
            gene_lookup: lookup::Lookup::new(),
        }
    }

    pub fn add_gene2(&mut self, gene_key: GeneKey, coordinates: u32) {
        self.gene_lookup.add(coordinates, gene_key).unwrap();
    }

    pub fn lookup_gene_id(&self, genes: &DenseSlotMap<GeneKey, Gene>, coordinates: u32) -> u32 {
        genes[*self.gene_lookup.find(coordinates)].id
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use rand::SeedableRng;

//     #[test]
//     fn test_gene_id() {
//         let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();

//         let mut cell = Cell::new();
//         let mut rng =
//             rand_pcg::Pcg32::from_seed([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
//         let gene1_key = cell.add_gene(&mut genes, &[3, 4], &mut rng);
//         let gene1_id = genes[gene1_key].id;

//         let gene2_key = cell.add_gene(&mut genes, &[5, 3], &mut rng);
//         let gene2_id = genes[gene2_key].id;

//         let found_gene1_key = cell.get_gene_key(gene1_id).unwrap();
//         assert_eq!(found_gene1_key, gene1_key);

//         let found_gene2_key = cell.get_gene_key(gene2_id).unwrap();
//         assert_eq!(found_gene2_key, gene2_key);

//         let lookup_gene_id = cell.lookup_gene_id(&genes, 3);
//         assert_eq!(gene1_id, lookup_gene_id);

//         let lookup_gene_id = cell.lookup_gene_id(&genes, 5);
//         assert_eq!(gene2_id, lookup_gene_id);
//     }
// }
