use bytemuck::Pod;
use std::marker::PhantomData;

/// Immutable slice
///
/// # Important Notes
///
/// - The struct transmust without checking endianness
#[derive(Clone)]
pub struct Slice<'a, T: Pod> {
    data: *const u8,
    stride: usize,
    count: usize,
    _phantom_data: PhantomData<&'a T>,
}

/// Mutable slice
///
/// # Important Notes
///
/// - The struct transmust without checking endianness
pub struct SliceMut<'a, T: Pod> {
    data: *mut u8,
    count: usize,
    stride: usize,
    _phantom_data: PhantomData<&'a mut T>,
}

macro_rules! impl_slice {
    ($ty:ident, $iter_name:ident) => {
        impl<'a, T: Pod> $ty<'a, T> {
            pub fn new(data: &[u8], count: usize, stride: usize) -> Self {
                Self {
                    data: data.as_ptr() as *mut u8,
                    count,
                    stride,
                    _phantom_data: PhantomData,
                }
            }

            pub fn new_with_offset<V: Pod>(data: &[V], offset: usize) -> Self {
                let bytes: &[u8] = bytemuck::cast_slice(data);
                Self::new(&bytes[offset..], data.len(), std::mem::size_of::<V>())
            }

            pub fn len(&self) -> usize {
                self.count
            }

            pub fn iter(&self) -> $iter_name<'a, T> {
                $iter_name {
                    data: self.data,
                    end: unsafe { self.data.offset((self.count * self.stride) as isize) },
                    stride: self.stride,
                    _phantom_data: PhantomData,
                }
            }
        }
    };
}
impl_slice!(Slice, SliceIterator);
impl_slice!(SliceMut, SliceMutIterator);

impl<'a, T: Pod> SliceMut<'a, T> {
    pub fn from_slice<V: Pod>(data: &mut [V]) -> Self {
        Self {
            data: data.as_mut_ptr() as *mut u8,
            count: data.len(),
            stride: std::mem::size_of::<V>(),
            _phantom_data: PhantomData,
        }
    }

    pub fn set<V: Pod>(&mut self, other_data: &[V]) {
        let other_stride = std::mem::size_of::<V>();
        assert!(
            other_stride <= std::mem::size_of::<T>(),
            "`data` type is {} bytes, but slice format expected at most {} bytes",
            std::mem::size_of::<V>(),
            std::mem::size_of::<T>()
        );

        let other_count = other_data.len();
        assert!(
            self.count <= self.len(),
            "`data` too large. Found slice with {} elements, but expected at most {}",
            other_count,
            self.count
        );

        let bytes: &[u8] = bytemuck::cast_slice(other_data);
        let other_ptr = bytes.as_ptr();
        for i in 0..self.count {
            let dst_start = self.stride * i;
            let src_start = i * other_stride;
            unsafe {
                // @todo: Document non-overlapping.
                self.data
                    .offset(src_start as isize)
                    .copy_from_nonoverlapping(other_ptr.offset(dst_start as isize), other_stride);
            }
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
                assert!(index < self.count, "index ouf of bounds");
                let start = self.stride * index;
                unsafe { std::mem::transmute::<_, &T>(self.data.offset(start as isize)) }
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
        assert!(index < self.count, "index ouf of bounds");
        let start = self.stride * index;
        unsafe { std::mem::transmute::<_, &mut T>(self.data.offset(start as isize)) }
    }
}

impl<'a, T: Pod> Slice<'a, T> {
    pub fn from_slice<V: Pod>(data: &[V]) -> Self {
        Self {
            data: data.as_ptr() as *const u8,
            count: data.len(),
            stride: std::mem::size_of::<V>(),
            _phantom_data: PhantomData,
        }
    }
}

///
/// Iterator
///

pub struct SliceIterator<'a, T: Pod> {
    data: *const u8,
    end: *const u8,
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

pub struct SliceMutIterator<'a, T: Pod> {
    data: *mut u8,
    end: *const u8,
    stride: usize,
    _phantom_data: PhantomData<&'a mut T>,
}

macro_rules! impl_iterator {
    ($ty: ident -> $type: ty) => {
        impl<'a, T: Pod> Iterator for $ty<'a, T> {
            type Item = $type;

            fn next(&mut self) -> Option<$type> {
                if self.data as *const u8 >= self.end {
                    return None;
                }
                let result: &mut T = unsafe { std::mem::transmute::<_, &mut T>(self.data) };
                self.data = unsafe { self.data.offset(self.stride as isize) };
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
            let slice: Slice<[f32; 3]> = Slice::from_slice(&vertices);
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [1.0, -1.0, 1.0]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 1.0, 0.0]);
            assert_eq!(iter.next(), None);
        }
        {
            let slice: Slice<[f32; 2]> =
                Slice::new_with_offset(&vertices, std::mem::size_of::<[f32; 3]>());
            let mut iter = slice.iter();
            assert_eq!(*iter.next().unwrap(), [0.25, 0.5]);
            assert_eq!(*iter.next().unwrap(), [-1.0, 0.25]);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn mutable_indexing() {
        let mut vertices = data();

        let mut slice: SliceMut<[f32; 3]> = SliceMut::from_slice(&mut vertices);
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
}
