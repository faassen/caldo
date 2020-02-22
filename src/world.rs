use rand::Rng;
use slotmap::DenseSlotMap;
use std::collections::HashMap;

use crate::cell::{Cell, CellKey};
use crate::gene::{Gene, GeneKey};
use crate::processor::{Config, Processor, ProcessorKey};

pub struct Entities {
    pub cells: DenseSlotMap<CellKey, Cell>,
    pub genes: DenseSlotMap<GeneKey, Gene>,
    pub processors: DenseSlotMap<ProcessorKey, Processor>,
    gene_by_id: HashMap<u32, GeneKey>,
}

pub struct World {
    pub entities: Entities,
    pub config: Config,
}

impl World {
    pub fn new(config: Config) -> World {
        World {
            entities: Entities {
                cells: DenseSlotMap::with_key(),
                genes: DenseSlotMap::with_key(),
                processors: DenseSlotMap::with_key(),
                gene_by_id: HashMap::new(),
            },
            config: config,
        }
    }

    pub fn create_cell(&mut self) -> CellKey {
        self.entities.cells.insert(Cell::new())
    }

    pub fn create_gene_in_cell<R: Rng>(
        &mut self,
        cell_key: CellKey,
        code: &[u32],
        rng: &mut R,
    ) -> GeneKey {
        self.entities.create_gene_in_cell(cell_key, code, rng)
    }

    pub fn create_gene(&mut self, code: &[u32]) -> GeneKey {
        self.entities.genes.insert(Gene::new(0, code))
    }

    pub fn create_processor(&mut self, cell_key: CellKey, gene_key: GeneKey) -> ProcessorKey {
        self.entities
            .processors
            .insert(Processor::new(cell_key, gene_key))
    }

    // pub fn execute_amount(&mut self, processor_key: ProcessorKey, amount: usize) {
    //     let processor = &self.entities.processors[processor_key];

    //     processor.execute_amount2(&mut self.entities, &self.config, amount);
    // }

    // pub fn execute_processors(&mut self, amount: usize) {
    //     for (_, processor) in &mut self.entities.processors {
    //         processor.execute_amount2(&mut self.entities, &self.config, amount);
    //     }
    // }
}

impl Entities {
    pub fn create_gene_id<R: Rng>(&self, rng: &mut R) -> u32 {
        loop {
            let id: u32 = rng.gen();
            if self.gene_by_id.contains_key(&id) {
                continue;
            }
            return id;
        }
    }

    pub fn get_gene_key(&self, cell_key: CellKey, gene_id: u32) -> Option<GeneKey> {
        match self.gene_by_id.get(&gene_id) {
            Some(&gene_key) => {
                let cell = &self.cells[cell_key];
                if cell.has_gene(gene_key) {
                    Some(gene_key)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn create_gene_in_cell<R: Rng>(
        &mut self,
        cell_key: CellKey,
        code: &[u32],
        rng: &mut R,
    ) -> GeneKey {
        let id = self.create_gene_id(rng);
        let gene = Gene::new(id, code);
        let coordinates = gene.coordinates();
        let gene_key = self.genes.insert(gene);
        self.gene_by_id.insert(id, gene_key);
        let cell = &mut self.cells[cell_key];
        cell.add_gene(gene_key, coordinates);
        gene_key
    }
}
