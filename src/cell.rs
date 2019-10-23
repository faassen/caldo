use crate::gene::Gene;
use crate::lookup;
use crate::processor::Processor;
use std::collections::HashMap;
extern crate rand_pcg;

use rand::{Rng, SeedableRng};
use std::rc::Rc;

// a world owns all cells, and all genes too

struct World<'a> {
    cell: Vec<Cell<'a>>,
    genes: HashMap<u32, Gene<'a>>,
}

struct Cell<'a> {
    genes: HashMap<u32, Rc<Gene<'a>>>,
    gene_lookup: lookup::Lookup<Rc<Gene<'a>>>,
    processors: Vec<Processor<'a>>,
}

impl<'a> Cell<'a> {
    fn new() -> Cell<'a> {
        Cell {
            genes: HashMap::new(),
            gene_lookup: lookup::Lookup::new(),
            processors: Vec::new(),
        }
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

    fn add_gene<R: Rng>(&mut self, gene: Gene<'a>, rng: &mut R) {
        let id = self.create_gene_id(rng);
        let rc_gene = Rc::new(gene);
        self.genes.insert(id, rc_gene);
        let rc_handle = self.genes.get(&id).unwrap();
        self.gene_lookup.add(Rc::clone(&rc_handle));
        // self.add_gene_lookup(&gene);
    }
}

// how to distinguish between genes if they're both stored
// at the same coordinate? we don't want to leave it up to the
// an implementation detail which one you get, and it shouldn't be random
// either.

// we don't want to burden accesses to genes with another index either,
// or an indirect gene handle - we want to maintain coordinate-based
// addressing.

// the simplest would be to place a gene *near* an existing gene so
// that each gene simply has a unique address. But this means that
// the instruction won't uniquely identify it, and we want content-based
// addressing.

// if there are two identical genes, I want to address the gene
// that was added latest, or the gene that wasn't. is a newest/oldest
// a way to address? it's definitely unique.

// a cell owns processors and genes

// a processor can reference a gene

// add gene

// find a gene by number of first instruction

// if a gene is destroyed, how do we clear its processors?
