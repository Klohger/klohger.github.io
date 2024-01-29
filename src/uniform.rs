use super::{f32x2, f32x3, f32x4, Mat2, Mat4, Quat};
use std::mem;

const _UNIFORM_CAST_CHECK: () = {
    if mem::size_of::<I32Uniform<2>>() != mem::size_of::<[i32; 2]>() {
        panic!()
    } else if mem::align_of::<I32Uniform<2>>() != mem::align_of::<[i32; 2]>() {
        panic!()
    }
};

#[repr(transparent)]
pub(crate) struct I32Uniform<const N: usize>([i32; N]);

impl AsRef<I32Uniform<1>> for i32 {
    fn as_ref(&self) -> &I32Uniform<1> {
        unsafe { mem::transmute(self) }
    }
}

impl<const N: usize> AsRef<I32Uniform<N>> for [i32; N] {
    fn as_ref(&self) -> &I32Uniform<N> {
        unsafe { mem::transmute(self) }
    }
}
#[repr(transparent)]
pub(crate) struct F32Uniform<const N: usize>([f32; N]);

impl AsRef<F32Uniform<1>> for f32 {
    fn as_ref(&self) -> &F32Uniform<1> {
        unsafe { mem::transmute(self) }
    }
}

impl<const N: usize> AsRef<F32Uniform<N>> for [f32; N] {
    fn as_ref(&self) -> &F32Uniform<N> {
        unsafe { mem::transmute(self) }
    }
}

#[repr(transparent)]
pub(crate) struct F32x2Uniform<const N: usize>([[f32; 2]; N]);

impl AsRef<F32x2Uniform<1>> for [f32; 2] {
    fn as_ref(&self) -> &F32x2Uniform<1> {
        unsafe { mem::transmute(self) }
    }
}

impl<const N: usize> AsRef<F32x2Uniform<N>> for [[f32; 2]; N] {
    fn as_ref(&self) -> &F32x2Uniform<N> {
        unsafe { mem::transmute(self) }
    }
}

impl AsRef<F32x2Uniform<1>> for f32x2 {
    fn as_ref(&self) -> &F32x2Uniform<1> {
        unsafe { mem::transmute(self) }
    }
}

#[repr(transparent)]
pub(crate) struct F32x3Uniform<const N: usize>([[f32; 3]; N]);

impl AsRef<F32x3Uniform<1>> for [f32; 3] {
    fn as_ref(&self) -> &F32x3Uniform<1> {
        unsafe { mem::transmute(self) }
    }
}

impl<const N: usize> AsRef<F32x3Uniform<N>> for [[f32; 3]; N] {
    fn as_ref(&self) -> &F32x3Uniform<N> {
        unsafe { mem::transmute(self) }
    }
}

impl AsRef<F32x3Uniform<1>> for f32x3 {
    fn as_ref(&self) -> &F32x3Uniform<1> {
        unsafe { mem::transmute(self) }
    }
}

#[repr(transparent)]
pub(crate) struct F32x4Uniform<const N: usize>([[f32; 4]; N]);

impl AsRef<F32x4Uniform<1>> for [f32; 4] {
    fn as_ref(&self) -> &F32x4Uniform<1> {
        unsafe { mem::transmute(self) }
    }
}

impl<const N: usize> AsRef<F32x4Uniform<N>> for [[f32; 4]; N] {
    fn as_ref(&self) -> &F32x4Uniform<N> {
        unsafe { mem::transmute(self) }
    }
}

impl<const N: usize> AsRef<F32x4Uniform<N>> for [f32x2; N] {
    fn as_ref(&self) -> &F32x4Uniform<N> {
        unsafe { mem::transmute(self) }
    }
}

impl<const N: usize> AsRef<F32x4Uniform<N>> for [f32x3; N] {
    fn as_ref(&self) -> &F32x4Uniform<N> {
        unsafe { mem::transmute(self) }
    }
}

impl AsRef<F32x4Uniform<1>> for f32x4 {
    fn as_ref(&self) -> &F32x4Uniform<1> {
        unsafe { mem::transmute(self) }
    }
}

impl<const N: usize> AsRef<F32x4Uniform<N>> for [f32x4; N] {
    fn as_ref(&self) -> &F32x4Uniform<N> {
        unsafe { mem::transmute(self) }
    }
}

#[repr(transparent)]
pub(crate) struct Mat2Uniform([f32; 2 * 2]);

impl AsRef<Mat2Uniform> for [[f32; 2]; 2] {
    fn as_ref(&self) -> &Mat2Uniform {
        unsafe { mem::transmute(self) }
    }
}

impl AsRef<Mat2Uniform> for [f32; 2 * 2] {
    fn as_ref(&self) -> &Mat2Uniform {
        unsafe { mem::transmute(self) }
    }
}

impl AsRef<Mat2Uniform> for Mat2 {
    fn as_ref(&self) -> &Mat2Uniform {
        unsafe { mem::transmute(self) }
    }
}

impl AsRef<Mat2Uniform> for Quat {
    fn as_ref(&self) -> &Mat2Uniform {
        unsafe { mem::transmute(self) }
    }
}

#[repr(transparent)]
pub(crate) struct Mat4Uniform([f32; 4 * 4]);

impl AsRef<Mat4Uniform> for [[f32; 4]; 4] {
    fn as_ref(&self) -> &Mat4Uniform {
        unsafe { mem::transmute(self) }
    }
}

impl AsRef<Mat4Uniform> for [f32; 4 * 4] {
    fn as_ref(&self) -> &Mat4Uniform {
        unsafe { mem::transmute(self) }
    }
}

impl AsRef<Mat4Uniform> for Mat4 {
    fn as_ref(&self) -> &Mat4Uniform {
        unsafe { mem::transmute(self) }
    }
}
