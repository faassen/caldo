use slotmap::DenseSlotMap;

use crate::cell::{Cell, CellKey};
use crate::gene::{Gene, GeneKey};

pub struct World {
    pub cells: DenseSlotMap<CellKey, Cell>,
    pub genes: DenseSlotMap<GeneKey, Gene>,
}

impl World {
    pub fn new() -> World {
        World {
            cells: DenseSlotMap::with_key(),
            genes: DenseSlotMap::with_key(),
        }
    }
}
