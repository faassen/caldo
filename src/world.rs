use slotmap::DenseSlotMap;

use crate::cell::{Cell, CellKey};
use crate::gene::{Gene, GeneKey};
use crate::processor::{Config, Processor, ProcessorKey};

pub struct Entities {
    pub cells: DenseSlotMap<CellKey, Cell>,
    pub genes: DenseSlotMap<GeneKey, Gene>,
    pub processors: DenseSlotMap<ProcessorKey, Processor>,
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
            },
            config: config,
        }
    }

    pub fn create_cell(&mut self) -> CellKey {
        self.entities.cells.insert(Cell::new())
    }

    pub fn create_gene(&mut self, code: &[u32]) -> GeneKey {
        self.entities.genes.insert(Gene::new(0, code))
    }

    pub fn create_processor(&mut self, cell_key: CellKey, gene_key: GeneKey) -> ProcessorKey {
        self.entities
            .processors
            .insert(Processor::new(cell_key, gene_key))
    }
}
