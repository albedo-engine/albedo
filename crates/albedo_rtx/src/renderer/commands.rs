pub trait Command {
    fn apply() -> ();
}

pub struct AddMesh<'a> {
    mesh: &'a Mesh,
    bvh: &'a BVH,
}

impl Command<'a> for AddMesh<'a> {}
