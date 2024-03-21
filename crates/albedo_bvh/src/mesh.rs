use albedo_backend::mesh::{AttributeId, IndexData, IndexDataSlice};
use strided_slice::Slice;

pub trait Mesh {
    fn indices(&self) -> Option<IndexDataSlice>;
    fn positions(&self) -> Option<Slice<[f32; 3]>>;
}

impl Mesh for albedo_backend::mesh::Primitive {
    fn indices(&self) -> Option<IndexDataSlice> {
        match &self.indices() {
            Some(IndexData::U16(v)) => Some(IndexDataSlice::U16(Slice::native(v))),
            Some(IndexData::U32(v)) => Some(IndexDataSlice::U32(Slice::native(v))),
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
