use std::ops::{Mul, Neg};

pub mod bsp;
pub mod texture;
pub mod wad;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Vec3(pub f32, pub f32, pub f32);

impl<'a> Mul for &'a Vec3 {
    type Output = f32;

    fn mul(self, rhs: &'a Vec3) -> f32 {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3(-self.0, -self.1, -self.2)
    }
}

pub fn read_struct<T>(buf: &[u8]) -> T {
    unsafe { std::ptr::read(buf.as_ptr() as *const _) }
}

pub fn read_mul_structs<T>(buf: &[u8]) -> Vec<T> {
    let size = std::mem::size_of::<T>();
    let count = buf.len() / size;

    let mut slice: Vec<T> = Vec::with_capacity(count);
    for i in 0..count {
        let s: T = read_struct(&buf[i * size..(i + 1) * size]);
        slice.push(s); // read whole code like memcpy
    }
    return slice;
}
