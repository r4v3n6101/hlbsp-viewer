const MAX_NAME: usize = 16;
const MIP_TEXTURES: usize = 4;

/// Representation of MipTex struct contains in bsp and wad files
#[repr(C)]
pub struct MipTex {
    pub name: [u8; MAX_NAME],
    pub width: u32,
    pub height: u32,
    pub offsets: [u32; MIP_TEXTURES],
}

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Texture {
    pub fn get(&self, x: u32, y: u32) -> u32 {
        return self.pixels[(self.width * y + x) as usize];
    }

    pub fn set(&mut self, x: u32, y: u32, color: u32) {
        self.pixels[(self.width * y + x) as usize] = color;
    }
}

impl MipTex {

    fn name_as_str<'a>(&'a self) -> &'a str {
        match std::ffi::CStr::from_bytes_with_nul(&self.name){
            Ok(val) => val.to_str().unwrap(),
            Err(_) => "null"
        }
    }

    /// Gather color table which is located after last mip texture and 2 bytes
    /// Required for building pixels from indices to color table
    /// Return slice to table consists of 256 RGB values or 256 * 3 bytes
    fn get_color_table<'a>(&self, mip_tex: &'a [u8]) -> &'a [u8] {
        let last_mip_offset = (self.offsets[3] + (self.width * self.height) / 64) as usize;
        let offset = last_mip_offset + 2; // 2 dummy bytes
        return &mip_tex[offset..offset + 256 * 3];
    }

    /// Read texture from mip_tex
    /// Require color table to access to RGB values by indices
    fn read_texture(&self, mip_tex: &[u8], col_table: &[u8], mip_level: u8) -> Texture {
        use std::mem::transmute;

        let mip_k = 1 << mip_level; // 2^mip_level
        let width = self.width as usize / mip_k;
        let height = self.height as usize / mip_k;

        let len = width * height;
        let mip_tex_offset = self.offsets[mip_level as usize] as usize;
        let indices_table = &mip_tex[mip_tex_offset..mip_tex_offset + len];

        let mut pixels: Vec<u32> = Vec::with_capacity(len);
        let transparent = self.name[0] == 0x7B; // '{' means that blue pixel hasn't color
        for i in indices_table {
            let i = *i as usize;
            let r = col_table[i * 3];
            let g = col_table[i * 3 + 1];
            let b = col_table[i * 3 + 2];
            let a: u8 = if transparent && r == 0 && g == 0 && b == 255 {
                0
            } else {
                255
            };
            let b = if a == 255 { b } else { 0 };
            let color = unsafe { transmute((r, g, b, a)) };
            pixels.push(color);
        }
        return Texture {
            width: width as u32,
            height: height as u32,
            pixels,
        };
    }

    pub fn get_name(&self) -> String {
        self.name_as_str().to_string()
    }

    pub fn get_texture(&self, mip_tex: &[u8], mip_level: u8) -> Texture {
        let color_table = self.get_color_table(mip_tex);
        let tex = self.read_texture(mip_tex, color_table, mip_level);
        return tex;
    }
}
