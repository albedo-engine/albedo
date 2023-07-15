use bytemuck::Pod;
use std::{marker::PhantomData, slice::SliceIndex};

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
                let start_byte = self.stride * index;
                assert!(
                    start_byte + std::mem::size_of::<T>() <= self.data.len(),
                    "index ouf of bounds"
                );
                let cast: &[T] = bytemuck::cast_slice(
                    &self.data[start_byte..start_byte + std::mem::size_of::<T>()],
                );
                &cast[0]
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
        let start_byte = self.stride * index;
        assert!(
            start_byte + std::mem::size_of::<T>() <= self.data.len(),
            "index ouf of bounds"
        );
        let cast: &mut [T] = bytemuck::cast_slice_mut(
            &mut self.data[start_byte..start_byte + std::mem::size_of::<T>()],
        );
        &mut cast[0]
    }
}
