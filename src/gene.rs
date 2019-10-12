use kdtree::distance::squared_euclidean;
use kdtree::ErrorKind;
use kdtree::KdTree;

pub struct Gene<'a> {
    pub code: &'a [u32],
}

impl<'a> Gene<'a> {
    pub fn new(code: &[u32]) -> Gene {
        return Gene { code: code };
    }
}

pub struct GeneLookup<'a> {
    tree: KdTree<f32, &'a Gene<'a>, [f32; 3]>,
}
