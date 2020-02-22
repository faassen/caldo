use slotmap::DenseSlotMap;

use crate::cell::{Cell, CellKey};
use crate::gene::{Gene, GeneKey};
use crate::processor::Processor;

pub struct World {
    pub cells: DenseSlotMap<CellKey, Cell>,
    pub genes: DenseSlotMap<GeneKey, Gene>,
    pub processors: DenseSlotMap<GeneKey, Processor>,
}

impl World {
    pub fn new() -> World {
        World {
            cells: DenseSlotMap::with_key(),
            genes: DenseSlotMap::with_key(),
            processors: DenseSlotMap::with_key(),
        }
    }

    pub fn create_cell(&mut self) -> CellKey {
        self.cells.insert(Cell::new())
    }
}
