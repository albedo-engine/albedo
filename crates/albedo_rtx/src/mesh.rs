pub trait Mesh {
    fn get_indices<'a>(&'a self) -> &'a [u32];
    fn get_position<'a>(&'a self, index: u32) -> Option<&'a [f32; 3]>;
}
