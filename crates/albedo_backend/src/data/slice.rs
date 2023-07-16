use bytemuck::Pod;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct Slice<'a, T: Pod> {
    data: &'a [u8],
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

pub struct SliceMut<'a, T: Pod> {
    data: &'a mut [u8],
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

pub struct SliceIterator<'a, T: Pod> {
    slice: &'a Slice<'a, T>,
    index: usize,
}

pub struct SliceMutIterator<'a, T: Pod> {
    slice: SliceMut<'a, T>,
    index: usize,
}

macro_rules! impl_slice {
    ($ty:ident) => {
        impl<'a, T: Pod> $ty<'a, T> {
            pub fn len(&self) -> usize {
                self.data.len() / self.stride
            }
        }
    };
}

impl_slice!(Slice);
impl_slice!(SliceMut);

impl<'a, T: Pod> SliceMut<'a, T> {
    pub fn set<V: Pod>(&mut self, data: &[V]) {
        let other_stride = std::mem::size_of::<V>();
        assert!(
            other_stride <= std::mem::size_of::<T>(),
            "`data` type is {} bytes, but slice format expected at most {} bytes",
            std::mem::size_of::<V>(),
            std::mem::size_of::<T>()
        );

        let count = self.len();
        let other_count = data.len();
        assert!(
            count <= self.len(),
            "`data` too large. Found slice with {} elements, but expected at most {}",
            other_count,
            count
        );

        let bytes: &[u8] = bytemuck::cast_slice(data);
        for i in 0..count {
            let dst_start = self.stride * i;
            let src_start = i * other_stride;
            self.data[dst_start..dst_start + other_stride]
                .copy_from_slice(&bytes[src_start..src_start + other_stride]);
        }
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
                let start = self.stride * index;
                let end = start + std::mem::size_of::<T>();
                assert!(end <= self.data.len(), "index ouf of bounds");
                bytemuck::from_bytes(&self.data[start..end])
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
        let start = self.stride * index;
        let end = start + std::mem::size_of::<T>();
        assert!(end <= self.data.len(), "index ouf of bounds");
        bytemuck::from_bytes_mut(&mut self.data[start..end])
    }
}

impl<'a, T: Pod> Iterator for Slice<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() == 0 {
            return None;
        }
        let result: &T = bytemuck::from_bytes(&self.data[0..std::mem::size_of::<T>()]);
        self.data = &self.data[self.stride..];
        Some(result)
    }
}

impl<'a, T: Pod> Iterator for SliceIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index = index + 1;
        Some(&self.slice[index])
    }
}
