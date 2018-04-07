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
    // TODO : Comments about start of buf
    pub fn get_color_table<'a>(&self, buf: &'a [u8]) -> &'a [u8] {
        let last_mip_offset = (self.offsets[3] + (self.width * self.height) / 64) as usize;
        let offset = last_mip_offset + 2; // 2 dummy bytes
        return &buf[offset..offset + 256 * 3];
    }

    pub fn read_texture(&self, buf: &[u8], col_table: &[u8], mip_level: usize) -> Vec<u8> { // TODO : Texture starts with '{' (0,0,255) is alpha
        let mip_tex_offset = self.offsets[mip_level] as usize;
        let mip_k = 1 << (2 * mip_level); // 2^2i, 1 is for zero level, 4 for 1nd and etc.
        let len = (self.width * self.height) as usize / mip_k;
        let indices_table = &buf[mip_tex_offset..mip_tex_offset + len];

        let mut colors: Vec<u8> = Vec::with_capacity(len * 3);
        for i in indices_table {
            let i = *i as usize;

            colors.push(col_table[i * 3]);
            colors.push(col_table[i * 3 + 1]);
            colors.push(col_table[i * 3 + 2]);
        }

        return colors;
    }
}