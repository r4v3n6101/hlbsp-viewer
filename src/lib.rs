use std::ops::{Mul, Neg};

pub mod bsp;
pub mod wad;
pub mod texture;

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
    use std::mem::{size_of, uninitialized};
    use std::slice::from_raw_parts_mut;

    let mut s: T = unsafe { uninitialized() };
    let size = size_of::<T>();
    unsafe {
        let s_data: &mut [u8] = from_raw_parts_mut(&mut s as *mut _ as *mut u8, size);
        s_data.copy_from_slice(&buf[0..size]);
    }
    return s;
}

pub fn read_mul_structs<T>(buf: &[u8]) -> Vec<T> {
    use std::mem::{size_of, uninitialized};
    use std::slice::from_raw_parts_mut;

    let size = size_of::<T>();
    let count = buf.len() / size;

    let mut slice: Vec<T> = Vec::with_capacity(count);
    for i in 0..count {
        let element_offset = i * size;
        let mut s: T = unsafe { uninitialized() };
        unsafe {
            let s_data: &mut [u8] = from_raw_parts_mut(&mut s as *mut _ as *mut u8, size);
            s_data.copy_from_slice(&buf[element_offset..element_offset + size]);
        }
        slice.push(s);
    }
    return slice;
}