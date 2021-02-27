pub trait Command {

    fn apply() -> ();

}

pub struct AddMesh<'a> {
    mesh: &'a Mesh,
    bvh: &'a BVH,
}
