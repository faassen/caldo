use kdtree::distance::squared_euclidean;
use kdtree::ErrorKind;
use kdtree::KdTree;

pub struct Lookup<T> {
    tree: KdTree<f32, T, [f32; 3]>,
}

impl<T> Lookup<T> {
    pub fn new() -> Lookup<T> {
        Lookup {
            tree: KdTree::new(3),
        }
    }

    pub fn add(&mut self, coordinates: u32, item: T) -> LookupAddResult {
        self.tree.add(coordinates_to_distance(coordinates), item)?;
        Ok(())
    }

    pub fn find(&self, coordinates: u32) -> &T {
        let v = self
            .tree
            .nearest(&coordinates_to_distance(coordinates), 1, &squared_euclidean)
            .unwrap();
        v[0].1
    }
}

fn coordinates_to_distance(nr: u32) -> [f32; 3] {
    let i = nr as u32;
    return [
        (i >> 16 & 0xff) as f32,
        (i >> 8 & 0xff) as f32,
        (i & 0xff) as f32,
    ];
}

#[derive(Debug, Clone, Copy)]
pub struct LookupError {}

pub type LookupAddResult = Result<(), LookupError>;

impl From<ErrorKind> for LookupError {
    fn from(_error: ErrorKind) -> Self {
        LookupError {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq)]
    struct Item {
        c: u32,
    }

    #[test]
    fn test_lookup_identify() -> LookupAddResult {
        let mut l = Lookup::<Item>::new();
        let i = Item { c: 0x010101 };
        l.add(i.c, i)?;
        assert_eq!(l.find(i.c), &i);
        return Ok(());
    }

    #[test]
    fn test_lookup_near() -> LookupAddResult {
        let mut l = Lookup::<Item>::new();
        let i1 = Item { c: 0x010101 };
        let i2 = Item { c: 0xF0F0F0 };

        l.add(i1.c, i1)?;
        l.add(i2.c, i2)?;
        assert_eq!(l.find(0x020202), &i1);
        return Ok(());
    }
}
