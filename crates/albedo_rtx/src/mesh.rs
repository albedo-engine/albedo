pub struct VertexData {
    position: [f32; 3],
    normal: [f32; 3],
    texcoords: [f32; 2],
}
pub trait Mesh {

    fn get_indices() -> &[u32];

    fn get_vertex_data(index: u32) -> VertexData;

}
