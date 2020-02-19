use crate::gene::Gene;
use crate::lookup;
use std::collections::HashMap;
extern crate rand_pcg;

use rand::Rng;
use std::rc::Rc;

// a world owns all cells

struct World {
    cell: Vec<Cell>,
}

pub struct Cell {
    genes: HashMap<u32, Rc<Gene>>,
    gene_lookup: lookup::Lookup<Rc<Gene>>,
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

    pub fn add_gene<R: Rng>(&mut self, code: &[u32], rng: &mut R) -> Rc<Gene> {
        let id = self.create_gene_id(rng);
        let gene = Gene::new(id, code);
        let rc_gene = Rc::new(gene);
        self.genes.insert(id, rc_gene);
        let rc_handle = self.genes.get(&id).unwrap();
        self.gene_lookup.add(Rc::clone(&rc_handle)).unwrap();
        Rc::clone(&rc_handle)
    }

    fn create_gene_id<R: Rng>(&self, rng: &mut R) -> u32 {
        loop {
            let id: u32 = rng.gen();
            if self.genes.contains_key(&id) {
                continue;
            }
            return id;
        }
    }

    pub fn lookup_gene_id(&self, coordinates: u32) -> u32 {
        self.gene_lookup.find(coordinates).id
    }

    pub fn get_gene(&self, gene_id: u32) -> Option<Rc<Gene>> {
        self.genes.get(&gene_id).map(|gene| Rc::clone(gene))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_gene_id() {
        let mut cell = Cell::new();
        let mut rng =
            rand_pcg::Pcg32::from_seed([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let gene1_id;
        {
            let gene1 = cell.add_gene(&[3, 4], &mut rng);
            gene1_id = gene1.id;
        }
        let gene2_id;
        {
            let gene2 = cell.add_gene(&[5, 3], &mut rng);
            gene2_id = gene2.id;
        }
        let gene1 = cell.get_gene(gene1_id).unwrap();
        assert_eq!(gene1.id, gene1_id);

        let gene2 = cell.get_gene(gene2_id).unwrap();
        assert_eq!(gene2.id, gene2_id);

        let lookup_gene_id = cell.lookup_gene_id(3);
        assert_eq!(gene1_id, lookup_gene_id);

        let lookup_gene_id = cell.lookup_gene_id(5);
        assert_eq!(gene2_id, lookup_gene_id);
    }
}
