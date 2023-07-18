use albedo_backend::{
    data::Slice,
    mesh::{AttributeId, IndexData, IndexDataSlice},
};

pub trait Mesh {
    fn indices(&self) -> Option<IndexDataSlice>;
    fn positions(&self) -> Option<Slice<[f32; 3]>>;
}

impl Mesh for albedo_backend::mesh::Primitive {
    fn indices(&self) -> Option<IndexDataSlice> {
        match &self.indices() {
            Some(IndexData::U16(v)) => Some(IndexDataSlice::U16(Slice::from_slice(v))),
            Some(IndexData::U32(v)) => Some(IndexDataSlice::U32(Slice::from_slice(v))),
            _ => None,
        }
    }

    fn positions(&self) -> Option<Slice<[f32; 3]>> {
        if let Some(index) = self.attribute_index(AttributeId::POSITION) {
            Some(self.attribute_f32x3(index))
        } else {
            None
        }
    }
}
