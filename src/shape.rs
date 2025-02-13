use crate::GGmlType;

use super::error::{GError, GResult, ShapeErrorKind};
use crate::GS_BLCK_SIZE;
pub const MAX_DIM: usize = 4;

pub(crate) fn zip<I, J>(i: I, j: J) -> std::iter::Zip<I::IntoIter, J::IntoIter>
where
    I: IntoIterator,
    J: IntoIterator,
{
    i.into_iter().zip(j)
}

#[derive(Copy, Clone)]
pub struct Axis(pub usize);

impl Axis {
    pub fn index(&self) -> usize {
        self.0
    }
}

#[inline(always)]
pub fn compute_stride_offset(n: usize, stride: usize) -> isize {
    (n * stride) as isize
}

#[derive(Clone, Debug)]
pub struct Dim {
    pub(crate) n_dims: usize,
    pub(crate) shape: Shape,
    pub(crate) stride: Layout,
}

impl Dim {
    pub fn shape(&self) -> &[usize] {
        &self.shape.as_slice()[..self.n_dims]
    }

    pub fn ret_stride(&mut self, stride: Layout) {
        self.stride = stride
    }

    // [a, b, c] => strides [b * c, c, 1]
    pub fn nd_stride(&self) -> Layout {
        let mut x: [usize; 4] = [0usize; 4];
        let s = self.shape().iter().rev();
        let mut prod = 1;
        let mut temp = 1;
        for (m, dim) in x[..self.n_dims].iter_mut().rev().zip(s) {
            prod *= temp;
            *m = prod;
            temp = *dim;
        }
        x
    }

    pub fn shape_mut(&mut self) -> &mut [usize] {
        &mut self.shape.as_slice_mut()[..self.n_dims]
    }

    pub fn shape_layout_mut(&mut self) -> &mut Layout {
        self.shape.layout_mut()
    }

    pub fn shape_layout(&self) -> &Layout {
        self.shape.layout()
    }

    pub fn stride_layout_mut(&mut self) -> &mut Layout {
        &mut self.stride
    }

    pub fn stride_layout(&self) -> &Layout {
        &self.stride
    }

    pub fn stride(&self) -> &[usize] {
        &self.stride[..self.n_dims]
    }

    pub fn stride_4d(&self) -> (usize, usize, usize, usize) {
        dims4(&self.stride)
    }

    pub fn stride_3d(&self) -> (usize, usize, usize) {
        dims3(&self.stride)
    }

    pub fn stride_2d(&self) -> (usize, usize) {
        dims2(&self.stride)
    }

    pub fn stride_1d(&self) -> usize {
        dims1(&self.stride)
    }

    pub fn stride_3(&self) -> usize {
        self.stride_layout()[3]
    }

    pub fn stride_2(&self) -> usize {
        self.stride_layout()[2]
    }

    pub fn stride_1(&self) -> usize {
        self.stride_layout()[1]
    }

    pub fn stride_0(&self) -> usize {
        self.stride_layout()[0]
    }

    pub fn dim_3(&self) -> usize {
        self.shape_layout()[3]
    }

    pub fn dim_2(&self) -> usize {
        self.shape_layout()[2]
    }

    pub fn dim_1(&self) -> usize {
        self.shape_layout()[1]
    }

    pub fn dim_0(&self) -> usize {
        self.shape.dims1()
    }

    pub fn dim4(&self) -> (usize, usize, usize, usize) {
        self.shape.dims4()
    }

    pub fn dim3(&self) -> (usize, usize, usize) {
        self.shape.dims3()
    }

    pub fn dim2(&self) -> (usize, usize) {
        self.shape.dims2()
    }

    #[inline]
    pub fn dim1(&self) -> usize {
        self.shape.dims1()
    }

    #[inline]
    pub fn n_dims(&self) -> usize {
        self.n_dims
    }

    #[inline]
    pub fn nrows(&self) -> usize {
        let (_, d1, d2, d3) = self.dim4();
        d1 * d2 * d3
    }

    pub fn is_vector(&self) -> bool {
        let (_, d1, d2, d3) = self.dim4();
        return d1 == 1 && d2 == 1 && d3 == 1;
    }

    pub(crate) fn elem_count(&self) -> usize {
        self.shape.elem_count()
    }

    // pub fn broadcast_with(&self, s: &Shape) -> GResult<Dim> {
    //     let stride = match crate::broadcast::upcast(s, &self.s, &self.stride) {
    //         Some(st) => st,
    //         None => return Err(GError::ShapeError(ShapeErrorKind::IncompatibleShape)),
    //     };
    //     Ok(Dim {
    //         s: s.clone(),
    //         stride: stride,
    //     })
    // }

    pub(crate) fn narrow(&self, dim: usize, start: usize, len: usize) -> GResult<Self> {
        let dims = self.shape();
        if dim >= dims.len() {
            return Err(GError::DimOutOfRange {
                shape: self.shape.clone(),
                dim: dim,
                op: "narrow",
            }
            .into());
        }
        if start + len > dims[dim] {
            return Err(GError::NarrowInvalidArgs {
                shape: self.shape.clone(),
                dim,
                start,
                len,
                msg: "start + len > dim_len",
            }
            .into());
        }
        let mut dims = dims.to_vec();
        dims[dim] = len;
        Ok(Self {
            n_dims: dims.len(),
            shape: Shape::from_vec(dims),
            stride: self.stride.clone(),
        })
    }

    pub fn transpose(&mut self, d1: usize, d2: usize) -> GResult<Dim> {
        let rank = self.shape.size();
        if rank <= d1 || rank <= d2 {
            return Err(GError::UnexpectedNumberOfDims {
                expected: usize::max(d1, d1),
                got: rank,
                shape: self.shape.clone(),
            }
            .into());
        }
        let mut stride = self.stride.clone();
        let dims = self.shape_mut();
        dims.swap(d1, d2);
        stride.swap(d1, d2);
        Ok(Self {
            n_dims: dims.len(),
            shape: Shape::from_slice(dims),
            stride: stride,
        })
    }

    //内存是否连续
    pub fn is_contiguous(&self) -> bool {
        // let (nb0, nb1, nb2, nb3) = self.stride_4d();
        // let (ne0, ne1, ne2, ne3) = self.dim4();
        // return nb0 == 1 && nb1 == nb0 * ne0 && nb2 == nb1 * ne1 && nb3 == nb2 * ne2;

        // tensor->nb[0] == GGML_TYPE_SIZE[tensor->type] &&
        // tensor->nb[1] == tensor->nb[0]*tensor->ne[0] &&
        // tensor->nb[2] == tensor->nb[1]*tensor->ne[1] &&
        // tensor->nb[3] == tensor->nb[2]*tensor->ne[2];
        let stride = self.stride();
        if self.shape().len() != stride.len() {
            return false;
        }
        let mut acc = 1;
        for (&stride, &dim) in stride.iter().zip(self.shape().iter()).rev() {
            if stride != acc {
                return false;
            }
            acc *= dim;
        }
        true
    }

    //内存是否连续
    pub fn ggml_is_contiguous(&self) -> bool {
        return self.stride[0] == 1
            && self.stride[1] == self.stride[0] * self.shape.layout()[0]
            && self.stride[2] == self.stride[1] * self.shape.layout()[1]
            && self.stride[3] == self.stride[2] * self.shape.layout()[2];
    }

    pub fn is_permuted(&self) -> bool {
        return self.stride[0] > self.stride[1]
            || self.stride[1] > self.stride[2]
            || self.stride[2] > self.stride[3];
    }

    pub fn is_scalar(&self) -> bool {
        let (ne0, ne1, ne2, ne3) = self.dim4();
        return ne0 == 1 && ne1 == 1 && ne2 == 1 && ne3 == 1;
    }

    pub(crate) fn strided_blocks(&self) -> crate::StridedBlocks {
        let mut block_len = 1;
        let mut contiguous_dims = 0; // These are counted from the right.
        for (&stride, &dim) in self.stride().iter().zip(self.shape().iter()).rev() {
            if stride != block_len {
                break;
            }
            block_len *= dim;
            contiguous_dims += 1;
        }
        let index_dims = self.shape().len() - contiguous_dims;
        if index_dims == 0 {
            crate::StridedBlocks::SingleBlock {
                start_offset: 0,
                len: block_len,
            }
        } else {
            let block_start_index = crate::StridedIndex::new(
                &self.shape()[..index_dims],
                &self.stride()[..index_dims],
                0,
            );
            crate::StridedBlocks::MultipleBlocks {
                block_start_index,
                block_len,
            }
        }
    }
}

trait One {
    fn one() -> Self;
}

pub type Layout = [usize; MAX_DIM];

impl One for Layout {
    fn one() -> Self {
        [1usize; MAX_DIM]
    }
}

#[derive(Clone, Debug)]
pub struct Shape(pub(crate) Layout);

impl Shape {
    pub fn from_vec(v: Vec<usize>) -> Shape {
        assert!(v.len() <= MAX_DIM);
        let mut s = Layout::one();
        s.iter_mut().zip(&v).for_each(|(si, vi)| *si = *vi);
        Shape(s)
    }

    pub fn layout(&self) -> &Layout {
        &self.0
    }

    pub fn layout_mut(&mut self) -> &mut Layout {
        &mut self.0
    }

    pub fn from_array<const N: usize>(v: [usize; N]) -> Shape {
        Shape::from_vec(v.to_vec())
    }

    pub fn from_slice(v: &[usize]) -> Shape {
        Shape::from_vec(v.to_vec())
    }

    pub fn dims4(&self) -> (usize, usize, usize, usize) {
        dims4(&self.0)
    }

    pub fn dims3(&self) -> (usize, usize, usize) {
        dims3(&self.0)
    }

    pub fn dims2(&self) -> (usize, usize) {
        dims2(&self.0)
    }

    #[inline]
    pub fn dims1(&self) -> usize {
        dims1(&self.0)
    }

    pub fn elem_count(&self) -> usize {
        self.0.iter().product()
    }

    pub fn iter<'a>(&'a self, n: usize) -> ShapeIter<'a> {
        ShapeIter {
            dim: self,
            index: self.first_index(n).unwrap(),
            first: true,
        }
    }

    pub(crate) fn stride_contiguous(&self) -> Layout {
        // let mut stride: Vec<_> = self
        //     .0
        //     .iter()
        //     .rev()
        //     .scan(1, |prod, u| {
        //         let prod_pre_mult = *prod;
        //         *prod *= u;
        //         Some(prod_pre_mult)
        //     })
        //     .collect();
        // stride.reverse();
        // stride
        let mut stride = [0; MAX_DIM];
        let mut prod = 1;

        for (i, &u) in self.0.iter().rev().enumerate().take(4) {
            stride[3 - i] = prod;
            prod *= u;
        }
        stride
    }

    pub fn select_axis(&self, a: Axis) -> Shape {
        let dst = select_axis(self.0.as_ref(), a);
        Shape::from_array(dst)
    }

    fn as_slice(&self) -> &[usize] {
        &self.0.as_ref()
    }

    fn as_slice_mut(&mut self) -> &mut [usize] {
        self.0.as_mut()
    }

    pub(crate) fn dims(&self, n_dims: usize) -> &[usize] {
        &self.as_slice()[..n_dims]
    }

    fn dims_mut(&mut self, n_dims: usize) -> &mut [usize] {
        &mut self.as_slice_mut()[..n_dims]
    }

    pub fn size(&self) -> usize {
        self.0.iter().fold(1, |s, &a| s * a as usize)
    }

    pub fn zero(s: usize) -> Self {
        Shape::from_vec(vec![0usize; s])
    }

    pub fn ggml_stride(&self, dtype: GGmlType) -> [usize; 4] {
        let mut x: [usize; 4] = [0usize; 4];
        x[0] = 1;
        x[1] = self.0[0] / GS_BLCK_SIZE[dtype as usize];
        for i in 2..MAX_DIM {
            x[i] = x[i - 1] * self.0[i - 1];
        }
        x
    }

    pub fn nd_stride(&self, n_dims: usize) -> [usize; 4] {
        let mut x: [usize; 4] = [0usize; 4];
        //vec![0; self.dim()];
        let s = self.dims(n_dims).iter().rev();
        let mut prod = 1;
        let mut temp = 1;
        for (m, dim) in x[..n_dims].iter_mut().rev().zip(s) {
            prod *= temp;
            *m = prod;
            temp = *dim;
        }
        x
    }

    pub fn dim(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub(crate) fn first_index(&self, n_dims: usize) -> Option<Self> {
        let slice = self.dims(n_dims);
        for ax in slice.iter() {
            if *ax == 0 {
                return None;
            }
        }
        Some(Self::zero(slice.len()))
    }

    #[inline]
    pub(crate) fn next_for(&self, n_dims: usize, index: &mut Self) -> Option<()> {
        let mut done = false;
        for (&dim, ix) in zip(self.dims(n_dims), index.dims_mut(n_dims)).rev() {
            *ix += 1;
            if *ix == dim {
                *ix = 0;
            } else {
                done = true;
                break;
            }
        }
        if done {
            Some(())
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn stride_offset(index: &[usize], strides: &[usize]) -> isize {
        let mut offset = 0;
        for (&i, &s) in index.iter().zip(strides.iter()) {
            offset += compute_stride_offset(i, s);
        }
        offset
    }
}

#[inline]
pub fn dims1(s: &[usize]) -> usize {
    assert!(s.len() >= 1);
    s[0]
}

#[inline]
pub fn dims2(s: &[usize]) -> (usize, usize) {
    assert!(s.len() >= 2);
    (s[0], s[1])
}

#[inline]
pub fn dims3(s: &[usize]) -> (usize, usize, usize) {
    assert!(s.len() >= 3);
    (s[0], s[1], s[2])
}

#[inline]
pub fn dims4(s: &[usize]) -> (usize, usize, usize, usize) {
    assert!(s.len() >= 4);
    (s[0], s[1], s[2], s[3])
}

impl PartialEq<Shape> for Shape {
    fn eq(&self, other: &Shape) -> bool {
        if self.elem_count() != other.elem_count() {
            return false;
        }
        return self.as_slice() == other.as_slice();
    }
}

pub fn select_axis(src: &[usize], a: Axis) -> Layout {
    let mut dst: Layout = Layout::one(); //vec![0; src.len() - 1];
    dst[..a.index()].copy_from_slice(&src[..a.index()]);
    //  dst[a.index()..].copy_from_slice(&src[a.index() + 1..]);
    dst[a.index()..]
        .iter_mut()
        .zip(&src[a.index() + 1..])
        .for_each(|(si, vi)| *si = *vi);
    dst
}

pub struct ShapeIter<'a> {
    dim: &'a Shape,
    index: Shape,
    first: bool,
}

impl<'a> ShapeIter<'a> {
    pub(crate) fn next(&mut self, n_dims: usize) -> Option<&Shape> {
        if self.first {
            self.first = false;
            Some(&self.index)
        } else {
            match self.dim.next_for(n_dims, &mut self.index) {
                Some(()) => Some(&self.index),
                None => None,
            }
        }
    }

    // pub(crate) fn fold<B, F>(mut self, init: B, mut f: F) -> B
    // where
    //     Self: Sized,
    //     F: FnMut(B, &Shape) -> B,
    // {
    //     let mut accum = init;
    //     while let Some(x) = self.next() {
    //         accum = f(accum, x);
    //     }
    //     accum
    // }
}

// impl<'a> Iterator for ShapeIter<'a> {
//     type Item = &'a [usize];
//     fn next(&mut self) -> Option<Self::Item> {
//         self.n()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test] //[3, 2]
    fn test_tensor_transpose() {
        let d = [4usize, 3, 2, 1];
        let t = d.iter();
        //  t.fold(init, f)
        // let d2 = d.select_axis(Axis(0));
        // println!("{:?}", d2);
    }

    #[test]
    fn test_dyn_dim_select_axis() {
        let d = Shape::from_vec(vec![4usize, 3, 2, 2]);
        let s = d.stride_contiguous();
        println!("s:{:?}", s);
    }

    #[test]
    fn test_shape_iter() {
        let d = Shape::from_vec(vec![3, 2]);
        // let mut i = d.iter(2);
        let strides = &d.ggml_stride(GGmlType::F16);
        println!("strides:{:?}", strides);
        println!("old strides:{:?}", d.nd_stride(2));

        let dim = Dim {
            n_dims: 2,
            shape: d,
            stride: Layout::default(),
        };
        println!("nd strides:{:?}", dim.nd_stride());

        // let ggml_strides = &d.ggml_strides();
        // println!("strides:{:?}", ggml_strides);
        // while let Some(x) = i.next(2) {
        //     let offset = Shape::stride_offset(x.dims(2), strides);
        //     println!("{:?},{}", x, offset);
        // }
    }
}
