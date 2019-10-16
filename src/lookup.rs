use kdtree::distance::squared_euclidean;
use kdtree::ErrorKind;
use kdtree::KdTree;

pub trait Coordinates {
    fn coordinates(&self) -> u32;
}

pub struct Lookup<'a, T: Coordinates> {
    tree: KdTree<f32, &'a T, [f32; 3]>,
}

impl<'a, T: Coordinates> Lookup<'a, T> {
    pub fn new() -> Lookup<'a, T> {
        Lookup {
            tree: KdTree::new(3),
        }
    }

    pub fn add(&mut self, item: &'a T) -> LookupAddResult {
        self.tree.add(coordinates_to_distance(item.coordinates()), item)?;
        Ok(())
    }

    pub fn find(&self, coordinates: u32) -> &T {
        let v = self
            .tree
            .nearest(&coordinates_to_distance(coordinates), 1, &squared_euclidean)
            .unwrap();
        *v[0].1
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
        c: u32
    }

    impl Coordinates for Item {
        fn coordinates(&self) -> u32 {
            self.c
        }
    }

    #[test]
    fn test_lookup_identify() -> LookupAddResult {
        let mut l = Lookup::<Item>::new();
        let i = Item { c: 0x010101 };
        l.add(&i)?;
        assert_eq!(l.find(i.coordinates()), &i);
        return Ok(());
    }

    #[test]
    fn test_lookup_near() -> LookupAddResult {
        let mut l = Lookup::<Item>::new();
        let i1 = Item { c: 0x010101 };
        let i2 = Item { c: 0xF0F0F0 };

        l.add(&i1)?;
        l.add(&i2)?;
        assert_eq!(l.find(0x020202), &i1);
        return Ok(());
    }

}
