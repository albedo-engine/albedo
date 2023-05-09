use std::marker::PhantomData;

use bytemuck::Pod;

fn compute_stride(sizes: &[usize]) -> usize {
    let mut stride = 0;
    for size in sizes {
        stride += size;
    }
    stride
}

pub struct InterleavedVec {
    data: Vec<u8>,
    stride: usize,
    sizes: Vec<usize>,
}

impl InterleavedVec {
    pub fn from_raw_data(data: Vec<u8>, sizes: Vec<usize>) -> Self {
        Self {
            data,
            stride: compute_stride(&sizes),
            sizes,
        }
    }

    pub fn with_capacity(count: usize, sizes: Vec<usize>) -> Self {
        let stride = compute_stride(&sizes);
        Self {
            data: Vec::with_capacity(count * stride),
            stride,
            sizes,
        }
    }

    pub fn set<T: Pod>(&mut self, index: usize, element: T) -> &mut Self {
        if std::mem::size_of::<T>() != self.stride as usize {
            panic!("push() called with an element that has an unexpected stide");
        }
        let byte_start = index * self.stride;
        let output: &mut [T] =
            bytemuck::cast_slice_mut(&mut self.data[byte_start..byte_start + self.stride]);
        output[0] = element;
        self
    }

    pub fn push<T: Pod>(&mut self, element: T) -> &mut Self {
        if std::mem::size_of::<T>() != self.stride as usize {
            panic!("push() called with an element that has an unexpected stide");
        }
        self.data
            .extend_from_slice(bytemuck::cast_slice(&[element]));
        self
    }

    pub fn iter<'a, T: Pod>(&'a self, element: usize) -> Result<InterleavedIter<'a, &'a T>, ()> {
        let byte_size = self.sizes[element] as usize;
        if std::mem::size_of::<T>() != byte_size {
            return Err(());
        }
        Ok(InterleavedIter {
            data: &self.data,
            byte_offset: self.byte_offset_for(element),
            stride: self.stride as usize,
            _phantom_data: PhantomData,
        })
    }

    pub fn iter_mut<'a, T: Pod>(
        &'a self,
        element: usize,
    ) -> Result<InterleavedIter<'a, &'a mut T>, ()> {
        let byte_size = self.sizes[element] as usize;
        if std::mem::size_of::<T>() != byte_size {
            return Err(());
        }
        Ok(InterleavedIter {
            data: &self.data,
            byte_offset: self.byte_offset_for(element),
            stride: self.stride as usize,
            _phantom_data: PhantomData,
        })
    }

    pub fn byte_offset_for(&self, element: usize) -> usize {
        // Compute the original byte offset.
        let mut byte_offset = 0;
        for i in 0..element {
            byte_offset += self.sizes[i];
        }
        byte_offset
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    pub fn count(&self) -> usize {
        self.data.len() / self.stride as usize
    }
}

pub struct InterleavedIter<'a, T> {
    data: &'a [u8],
    byte_offset: usize,
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T> Iterator for InterleavedIter<'a, T>
where
    T: Pod,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let byte_size = std::mem::size_of::<T>();
        if self.byte_offset >= self.data.len() {
            return None;
        }
        let cast: &[T] =
            bytemuck::cast_slice(&self.data[self.byte_offset..self.byte_offset + byte_size]);

        self.byte_offset += self.stride;
        Some(cast[0])
    }
}
