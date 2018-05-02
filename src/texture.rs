use read_struct;
pub use std::{collections::HashMap, path::Path};
use std::{fs::File, io::{BufReader, Read}};
use wad::{entries, read_name};

const MAX_NAME: usize = 16;
const MIP_TEXTURES: usize = 4;

#[repr(C)]
pub struct MipTex {
    pub name: [u8; MAX_NAME],
    pub width: u32,
    pub height: u32,
    pub offsets: [u32; MIP_TEXTURES],
}

pub struct Texture { pub width: u32, pub height: u32, pub pixels: Vec<u32> }

impl Texture {
    pub fn get(&self, x: u32, y: u32) -> u32 {
        return self.pixels[(self.width * y + x) as usize];
    }

    pub fn set(&mut self, x: u32, y: u32, color: u32) {
        self.pixels[(self.width * y + x) as usize] = color;
    }
}

impl MipTex {
    pub fn get_color_table<'a>(&self, mip_tex: &'a [u8]) -> &'a [u8] {
        let last_mip_offset = (self.offsets[3] + (self.width * self.height) / 64) as usize;
        let offset = last_mip_offset + 2; // 2 dummy bytes
        return &mip_tex[offset..offset + 256 * 3];
    }

    pub fn read_texture(&self, mip_tex: &[u8], col_table: &[u8], mip_level: usize) -> Texture {
        use std::mem::transmute;

        let mip_k = 1 << mip_level; // 2^mip_level
        let width = self.width as usize / mip_k;
        let height = self.height as usize / mip_k;

        let len = width * height;
        let mip_tex_offset = self.offsets[mip_level] as usize;
        let indices_table = &mip_tex[mip_tex_offset..mip_tex_offset + len];

        let mut pixels: Vec<u32> = Vec::with_capacity(len);
        let transparent = self.name[0] == 0x7B; // '{' means that blue pixel hasn't color
        for i in indices_table {
            let i = *i as usize;
            let r = col_table[i * 3];
            let g = col_table[i * 3 + 1];
            let b = col_table[i * 3 + 2];
            let a: u8 = if transparent && r == 0 && g == 0 && b == 255 { 0 } else { 255 };
            let b = if a == 255 { b } else { 0 };
            let color = unsafe { transmute((r, g, b, a)) };
            pixels.push(color);
        }
        return Texture { width: width as u32, height: height as u32, pixels };
    }
}

pub fn read_textures(path: &Path, mip_level: usize) -> HashMap<String, Texture> {
    fn read_file(path: &Path, mip_level: usize) -> HashMap<String, Texture> {
        let wad_file = File::open(path).expect("Error reading wad file");
        let size = wad_file.metadata().unwrap().len() + 1;
        let mut wad: Vec<u8> = Vec::with_capacity(size as usize);

        let mut buf_reader = BufReader::new(wad_file);
        buf_reader.read_to_end(&mut wad).unwrap();

        entries(&wad).iter().map(|entry| {
            let tex_offset = entry.file_pos as usize;
            let tex: MipTex = read_struct(&wad[tex_offset..]);

            let col_table = tex.get_color_table(&wad[tex_offset..]);
            let texture = tex.read_texture(&wad[tex_offset..], col_table, mip_level);

            let name = read_name(tex.name);
            (name, texture)
        }).collect()
    }
    if path.is_file() {
        read_file(path, mip_level)
    } else {
        path.read_dir().expect("Error while reading dir").flat_map(|entry| {
            let entry = entry.expect("Error while reading dir entry");
            read_file(&entry.path(), mip_level)
        }).collect()
    }
}