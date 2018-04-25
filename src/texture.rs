pub use std::collections::HashMap;

const MAX_NAME: usize = 16;
const MIP_TEXTURES: usize = 4;

#[repr(C)]
pub struct MipTex {
    pub name: [u8; MAX_NAME],
    pub width: u32,
    pub height: u32,
    pub offsets: [u32; MIP_TEXTURES],
}

pub fn read_name(name: [u8; MAX_NAME]) -> String {
    let index_null = name.iter().position(|&c| c == 0);
    let str = match index_null {
        Some(i) => &name[..i],
        None => &name,
    };
    return String::from_utf8_lossy(str).into_owned();
}

pub struct Texture { pub width: i32, pub height: i32, pub pixels: Vec<i32> }

impl Texture {
    pub fn get(&self, x: i32, y: i32) -> i32 {
        return self.pixels[(self.width * y + x) as usize];
    }

    pub fn set(&mut self, x: i32, y: i32, color: i32) {
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

        let mut pixels: Vec<i32> = Vec::with_capacity(len);
        let transparent = self.name[0] == 0x7B; // '{' means that blue pixel hasn't color
        for i in indices_table {
            let i = *i as usize;
            let r = col_table[i * 3];
            let g = col_table[i * 3 + 1];
            let b = col_table[i * 3 + 2];
            let a: u8 = if transparent && r == 0 && g == 0 && b == 255 { 0 } else { 255 };
            let color = unsafe { transmute((r, g, b, a)) };
            pixels.push(color);
        }
        return Texture { width: width as i32, height: height as i32, pixels };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn list_textures_test() {
        use std::fs::File;
        use wad::*;
        use texture::*;
        use std::io::*;
        use read_struct;
        use image::ImageBuffer;

        let name = "halflife.wad";
        let mut f: File = File::open(name).unwrap();
        let size = f.metadata().unwrap().len() as usize;

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        f.read_to_end(&mut buf).unwrap();

        let entries: Vec<DirEntry> = entries(&buf);
        let map: HashMap<String, Texture> = entries.iter().map(|entry| {
            let tex_offset: usize = entry.file_pos as usize;
            let tex: MipTex = read_struct(&buf[tex_offset..]);
            let col_table = tex.get_color_table(&buf[tex_offset..]);

            let name: String = read_name(tex.name);
            let texture: Texture = tex.read_texture(&buf[tex_offset..], col_table, 0);

            (name, texture)
        }).collect();

        let atlas = TextureAtlas::build_atlas(map);

        let req = atlas.texture;
        let mut imgbuf = ImageBuffer::new(req.width as u32, req.height as u32);
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            *pixel = ::image::Rgba(unsafe { ::std::mem::transmute::<_, [u8; 4]>(req.get(x as i32, y as i32)) });
        }
        ::image::ImageRgba8(imgbuf).save(&mut File::create("atlas.png").unwrap(), ::image::PNG).unwrap();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect { pub x: i32, pub y: i32, pub width: i32, pub height: i32 }

impl Rect {
    pub fn is_contained_in(&self, b: &Rect) -> bool {
        self.x >= b.x && self.y >= b.y
            && self.x + self.width <= b.x + b.width
            && self.y + self.height <= b.y + b.height
    }

    pub fn intersects(&self, b: &Rect) -> bool {
        (self.x - b.x).abs() * 2 < self.width + b.width &&
            (self.y - b.y).abs() * 2 < self.height + b.height
    }
}

pub struct TextureAtlas { pub texture: Texture, pub slots: HashMap<String, Rect> }

impl TextureAtlas {
    // TODO : Replace
    pub fn build_atlas(textures: HashMap<String, Texture>) -> TextureAtlas {
        let a: f64 = textures.iter().map(|(_, tex)| tex.width).sum::<i32>() as f64;
        let b: f64 = textures.iter().map(|(_, tex)| tex.height).sum::<i32>() as f64;
        let total_x: i32 = 1 << (a.log2() as i32 - 5); // TODO : width and height calculation heuristic
        let total_y: i32 = 1 << (b.log2() as i32 - 5);
        println!("Generating {}x{} atlas", total_x, total_y);
        let mut max_rects = MaxRects::init(total_x, total_y);
        let mut texture = Texture {
            width: total_x,
            height: total_y,
            pixels: vec![0; (total_x * total_y) as usize],
        };
        let mut slots: HashMap<String, Rect> = HashMap::with_capacity(textures.len());
        for (name, tex) in textures {
            let w = tex.width;
            let h = tex.height;
            match max_rects.place_rect_a(w, h) {
                Some(rect) => {
                    println!("Packing {}", name);
                    for i in 0..w {
                        for j in 0..h {
                            texture.set(rect.x + i, rect.y + j, tex.get(i, j));
                        }
                    }
                    slots.insert(name, rect);
                }
                None => {
                    println!("Not enough space for {}", name);
                }
            }
        }
        return TextureAtlas { texture, slots };
    }
}

pub struct MaxRects { pub width: i32, pub height: i32, pub free: Vec<Rect>, pub used: Vec<Rect> } // TODO : Replace with single method?

// TODO : more inlining and cleaning up
impl MaxRects {
    pub fn init(width: i32, height: i32) -> MaxRects {
        let slot = Rect { width, height, x: 0, y: 0 };
        Self {
            width,
            height,
            free: vec![slot],
            used: Vec::new(),
        }
    }

    // TODO : Rename
    pub fn place_rect_a(&mut self, width: i32, height: i32) -> Option<Rect> {
        let rect = self.find_best_area(width, height);
        if let Some(x) = rect {
            self.place_rect(x);
        }
        return rect;
    }

    pub fn occupancy(&self) -> f32 {
        let total_area: i32 = self.width * self.height;
        let used_area: i32 = self.used.iter().map(|rect| rect.width * rect.height).sum();

        return used_area as f32 / total_area as f32;
    }

    pub fn place_rect(&mut self, rect: Rect) {
        let mut size = self.free.len();
        let mut i = 0;
        while i < size {
            let free_rect = self.free[i];
            if !(rect.x >= free_rect.x + free_rect.width || rect.x + rect.width <= free_rect.x ||
                rect.y >= free_rect.y + free_rect.height || rect.y + rect.height <= free_rect.y) {
                self.split_free_node(free_rect, &rect);
                self.free.remove(i);
                size -= 1;
            } else {
                i += 1;
            }
        }
        self.prune_free_list();
        self.used.push(rect);
    }

    fn split_free_node(&mut self, free_node: Rect, used_node: &Rect) {
        if used_node.x < free_node.x + free_node.width && used_node.x + used_node.width > free_node.x {
            // New node at the top side of the used node.
            if used_node.y > free_node.y && used_node.y < free_node.y + free_node.height {
                let mut new_node = free_node;
                new_node.height = used_node.y - new_node.y;
                self.free.push(new_node);
            }

            // New node at the bottom side of the used node.
            if used_node.y + used_node.height < free_node.y + free_node.height {
                let mut new_node = free_node;
                new_node.y = used_node.y + used_node.height;
                new_node.height = free_node.y + free_node.height - (used_node.y + used_node.height);
                self.free.push(new_node);
            }
        }
        if used_node.y < free_node.y + free_node.height && used_node.y + used_node.height > free_node.y {
            // New node at the left side of the used node.
            if used_node.x > free_node.x && used_node.x < free_node.x + free_node.width {
                let mut new_node = free_node;
                new_node.width = used_node.x - new_node.x;
                self.free.push(new_node);
            }

            // New node at the right side of the used node.
            if used_node.x + used_node.width < free_node.x + free_node.width {
                let mut new_node = free_node;
                new_node.x = used_node.x + used_node.width;
                new_node.width = free_node.x + free_node.width - (used_node.x + used_node.width);
                self.free.push(new_node);
            }
        }
    }

    fn prune_free_list(&mut self) {
        let mut i = 0;
        while i < self.free.len() {
            let mut j = i + 1;
            while j < self.free.len() {
                if self.free[i].is_contained_in(&self.free[j]) {
                    self.free.remove(i);
                    i -= 1;
                    break;
                }
                if self.free[j].is_contained_in(&self.free[i]) {
                    self.free.remove(j);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }

    fn find_best_area(&self, width: i32, height: i32) -> Option<Rect> {
        use std::i32::MAX;
        let mut x = 0;
        let mut y = 0;
        let mut best_area_fit = MAX;
        let mut best_short_side_fit = MAX;
        for rect in &self.free {
            if rect.width >= width && rect.height >= height {
                let area_fit = rect.width * rect.height - width * height;
                let leftover_horiz = rect.width - width;
                let leftover_vert = rect.height - height;
                let short_side_fit = leftover_horiz.min(leftover_vert);

                if area_fit < best_area_fit ||
                    (area_fit == best_area_fit && short_side_fit < best_short_side_fit) {
                    best_area_fit = area_fit;
                    best_short_side_fit = short_side_fit;

                    x = rect.x;
                    y = rect.y;
                }
            }
        }
        return if best_area_fit == MAX { None } else { Some(Rect { x, y, width, height }) };
    }
}