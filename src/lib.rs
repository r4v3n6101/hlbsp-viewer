pub mod hlbsp;

#[repr(C)]
#[derive(Debug)]
pub struct Vec3(f32, f32, f32);

pub fn read_struct<S>(buf: &[u8], offset: usize) -> S {
    use std::mem::{size_of, uninitialized};
    use std::slice::from_raw_parts_mut;

    let mut s: S = unsafe { uninitialized() };
    let size = size_of::<S>();
    unsafe {
        let s_data: &mut [u8] = from_raw_parts_mut(&mut s as *mut _ as *mut u8, size);
        s_data.copy_from_slice(&buf[offset..offset + size]);
    }
    return s;
}

pub fn read_mul_struct<S>(buf: &[u8], offset: usize, length: usize) -> Vec<S> {
    use std::mem::{size_of, uninitialized};
    use std::slice::from_raw_parts_mut;
    let size = size_of::<S>();
    let count = length / size;

    let mut slice: Vec<S> = Vec::with_capacity(count);
    for i in 0..count {
        let element_offset = offset + i * size;
        let mut s: S = unsafe { uninitialized() };
        unsafe {
            let s_data: &mut [u8] = from_raw_parts_mut(&mut s as *mut _ as *mut u8, size);
            s_data.copy_from_slice(&buf[element_offset..element_offset + size]);
        }
        slice.push(s);
    }
    return slice;
}