use bytemuck::Pod;
use std::marker::PhantomData;

// @todo: Those structs could have better checks.

/// Immutable slice
///
/// # Important Notes
///
/// - The struct transmust without checking endianness
#[derive(Clone)]
pub struct Slice<'a, T: Pod> {
    data: &'a [u8],
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T: Pod> Slice<'a, T> {
    pub fn new(data: &'a [u8], stride: usize, offset: usize) -> Self {
        Self {
            data: &data[offset..],
            stride,
            _phantom_data: PhantomData,
        }
    }

    pub fn from_slice_offset<V: Pod>(data: &'a [V], offset: usize) -> Self {
        let bytes: &[u8] = bytemuck::cast_slice(data);
        Self::new(bytes, std::mem::size_of::<V>(), offset)
    }

    pub fn from_slice<V: Pod>(data: &'a [V]) -> Self {
        Self::from_slice_offset(data, 0)
    }

    pub fn len(&self) -> usize {
        self.data.len() / self.stride
    }

    pub fn iter(&'a self) -> SliceIterator<'a, T> {
        SliceIterator::new(self)
    }
}

/// Mutable slice
///
/// # Important Notes
///
/// - The struct transmust without checking endianness
pub struct SliceMut<'a, T: Pod> {
    data: &'a mut [u8],
    stride: usize,
    _phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T: Pod> SliceMut<'a, T> {
    pub fn new(data: &'a mut [u8], stride: usize, offset: usize) -> Self {
        Self {
            data: &mut data[offset..],
            stride,
            _phantom_data: PhantomData,
        }
    }

    pub fn from_slice_offset<V: Pod>(data: &'a mut [V], offset: usize) -> Self {
        let bytes: &mut [u8] = bytemuck::cast_slice_mut(data);
        Self::new(bytes, std::mem::size_of::<V>(), offset)
    }

    pub fn from_slice<V: Pod>(data: &'a mut [V]) -> Self {
        Self::from_slice_offset(data, 0)
    }

    pub fn set<V: Pod>(&mut self, other_data: &[V]) {
        let other_stride = std::mem::size_of::<V>();
        assert!(
            other_stride <= std::mem::size_of::<T>(),
            "`data` type is {} bytes, but slice format expected at most {} bytes",
            std::mem::size_of::<V>(),
            std::mem::size_of::<T>()
        );

        let count = self.len();
        let other_count = other_data.len();
        assert!(
            count <= self.len(),
            "`data` too large. Found slice with {} elements, but expected at most {}",
            other_count,
            count
        );

        let ptr = self.data.as_mut_ptr();
        let bytes: &[u8] = bytemuck::cast_slice(other_data);
        let other_ptr = bytes.as_ptr();
        for i in 0..count {
            let dst_start = self.stride * i;
            let src_start = i * other_stride;
            unsafe {
                // @todo: Document non-overlapping.
                ptr.offset(src_start as isize)
                    .copy_from_nonoverlapping(other_ptr.offset(dst_start as isize), other_stride);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.data.len() / self.stride
    }

    pub fn iter(&'a mut self) -> SliceMutIterator<'a, T> {
        SliceMutIterator::new(self)
    }
}

macro_rules! impl_indexing {
    ($ty:ident) => {
        impl<'a, T> std::ops::Index<usize> for $ty<'a, T>
        where
            T: Pod,
        {
            type Output = T;

            fn index(&self, index: usize) -> &Self::Output {
                assert!(index < self.len(), "index ouf of bounds");
                let start = self.stride * index;
                let ptr = self.data.as_ptr();
                unsafe { std::mem::transmute::<_, &T>(ptr.offset(start as isize)) }
            }
        }
    };
}
impl_indexing!(Slice);
impl_indexing!(SliceMut);

impl<'a, T> std::ops::IndexMut<usize> for SliceMut<'a, T>
where
    T: Pod,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len(), "index ouf of bounds");
        let start = self.stride * index;
        let ptr = self.data.as_mut_ptr();
        unsafe { std::mem::transmute::<_, &mut T>(ptr.offset(start as isize)) }
    }
}

///
/// Iterator
///

pub struct SliceIterator<'a, T: Pod> {
    data: &'a [u8],
    index: usize,
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T: Pod> SliceIterator<'a, T> {
    pub fn new(slice: &'a Slice<'a, T>) -> Self {
        Self {
            data: slice.data,
            index: 0,
            stride: slice.stride,
            _phantom_data: PhantomData,
        }
    }
}

pub struct SliceMutIterator<'a, T: Pod> {
    data: &'a mut [u8],
    index: usize,
    stride: usize,
    _phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T: Pod> SliceMutIterator<'a, T> {
    pub fn new(slice: &'a mut SliceMut<'a, T>) -> Self {
        Self {
            data: &mut slice.data,
            index: 0,
            stride: slice.stride,
            _phantom_data: PhantomData,
        }
    }
}

macro_rules! impl_iterator {
    ($ty: ident -> $type: ty) => {
        impl<'a, T: Pod> Iterator for $ty<'a, T> {
            type Item = $type;

            fn next(&mut self) -> Option<$type> {
                let offset = self.index * self.stride;
                if offset >= self.data.len() {
                    return None;
                }
                self.index = self.index + 1;
                let ptr = self.data.as_ptr();
                let result: &mut T =
                    unsafe { std::mem::transmute::<_, &mut T>(ptr.offset(offset as isize)) };
                Some(result)
            }
        }
    };
}

impl_iterator!(SliceIterator -> &'a T);
impl_iterator!(SliceMutIterator -> &'a mut T);

///
/// Test
///

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    #[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
    struct Vertex {
        position: [f32; 3],
        uv: [f32; 2],
    }

    fn data() -> Vec<Vertex> {
        vec![
            Vertex {
                position: [1.0, -1.0, 1.0],
                uv: [0.25, 0.5],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                uv: [-1.0, 0.25],
            },
        ]
    }

    #[test]
    fn slice_count() {
        let vertices = data();
        let slice: Slice<[f32; 3]> = Slice::from_slice(&vertices);
        assert_eq!(slice.len(), 2);
    }

    #[test]
    fn immutable_indexing() {
        let vertices = data();
        let slice: Slice<[f32; 3]> = Slice::from_slice(&vertices);
        assert_eq!(slice[0], [1.0, -1.0, 1.0]);
        assert_eq!(slice[1], [-1.0, 1.0, 0.0]);
    }

    #[test]
    fn immutable_iter() {
        let vertices = data();
        {
            let slice: Slice<[f32; 3]> = Slice::from_slice_offset(&vertices, 0);
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [1.0, -1.0, 1.0]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 1.0, 0.0]);
            assert_eq!(iter.next(), None);
        }
        {
            let slice: Slice<[f32; 2]> =
                Slice::from_slice_offset(&vertices, std::mem::size_of::<[f32; 3]>());
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [0.25, 0.5]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 0.25]);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn mutable_indexing() {
        let mut vertices = data();

        let mut slice: SliceMut<[f32; 3]> = SliceMut::from_slice_offset(&mut vertices, 0);
        assert_eq!(slice[0], [1.0, -1.0, 1.0]);
        assert_eq!(slice[1], [-1.0, 1.0, 0.0]);

        // Changing index 0 doesn't affect other index.
        slice[0] = [4.0, 3.0, 1.0];
        assert_eq!(slice[0], [4.0, 3.0, 1.0]);
        assert_eq!(slice[1], [-1.0, 1.0, 0.0]);

        // Changing index 1 doesn't affect other index.
        slice[1] = [11.0, 10.0, 9.0];
        assert_eq!(slice[0], [4.0, 3.0, 1.0]);
        assert_eq!(slice[1], [11.0, 10.0, 9.0]);
    }

    #[test]
    fn mutable_iter() {
        let mut vertices = data();
        {
            let mut slice: SliceMut<[f32; 3]> = SliceMut::from_slice_offset(&mut vertices, 0);
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [1.0, -1.0, 1.0]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 1.0, 0.0]);
            assert_eq!(iter.next(), None);
        }
        {
            let mut slice: SliceMut<[f32; 2]> =
                SliceMut::from_slice_offset(&mut vertices, std::mem::size_of::<[f32; 3]>());
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [0.25, 0.5]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 0.25]);
            assert_eq!(iter.next(), None);
        }
    }
}
