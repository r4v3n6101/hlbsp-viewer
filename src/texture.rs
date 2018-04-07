const MAX_NAME: usize = 16;
const MIP_TEXTURES: usize = 4;

#[repr(C)]
pub struct MipTex {
    pub name: [u8; MAX_NAME],
    pub width: u32,
    pub height: u32,
    pub offsets: [u32; MIP_TEXTURES],
}

impl MipTex {
    pub fn get_color_table<'a>(&self, mip_tex: &'a [u8]) -> &'a [u8] {
        let last_mip_offset = (self.offsets[3] + (self.width * self.height) / 64) as usize;
        let offset = last_mip_offset + 2; // 2 dummy bytes
        return &mip_tex[offset..offset + 256 * 3];
    }

    pub fn read_texture(&self, mip_tex: &[u8], col_table: &[u8], mip_level: usize) -> Vec<u8> {
        let mip_tex_offset = self.offsets[mip_level] as usize;
        let mip_k = 1 << (2 * mip_level); // 2^2i, 1 is for zero level, 4 for 1nd and etc.
        let len = (self.width * self.height) as usize / mip_k;
        let indices_table = &mip_tex[mip_tex_offset..mip_tex_offset + len];

        let mut colors: Vec<u8> = Vec::with_capacity(len * 4);
        let transparent = self.name[0] == 0x7B; // '{' means that the pixel (0,0,255) hasn't color
        for i in indices_table {
            let i = *i as usize;
            let r = col_table[i * 3];
            let g = col_table[i * 3 + 1];
            let b = col_table[i * 3 + 2];
            let a = if transparent && r == 0 && g == 0 && b == 255 { 0 } else { 255 };
            colors.push(r);
            colors.push(g);
            colors.push(b);
            colors.push(a);
        }

        return colors;
    }
}