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

pub struct Texture { pub width: u32, pub height: u32, pub pixels: Vec<i32> }

impl Texture {
    pub fn get(&self, x: u32, y: u32) -> i32 {
        return self.pixels[(self.width * y + x) as usize];
    }

    pub fn set(&mut self, x: u32, y: u32, color: i32) {
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
        return Texture { width: width as u32, height: height as u32, pixels };
    }
}

#[derive(Clone, Debug)]
pub struct Rect { pub x: u32, pub y: u32, pub width: u32, pub height: u32 }

impl Rect {
    pub fn is_contained_in(&self, b: &Rect) -> bool {
        self.x >= b.x && self.y >= b.y
            && self.x + self.width <= b.x + b.width
            && self.y + self.height <= b.y + b.height
    }

    pub fn intersects(&self, b: &Rect) -> bool {
        self.x < b.x + b.width && self.x + self.width > b.x &&
            self.y < b.y + b.height && self.y + self.height > b.y
    }
}

pub struct TextureAtlas { pub texture: Texture, pub slots: HashMap<String, Rect> }

impl TextureAtlas {
    pub fn occupancy(&self) -> f32 {
        let total_area: u32 = self.texture.width * self.texture.height;
        let used_area: u32 = self.slots.iter().map(|(_, rect)| rect.width * rect.height).sum();
        return used_area as f32 / total_area as f32;
    }
}

pub fn build_atlas(textures: HashMap<String, &Texture>) -> TextureAtlas {
    use std::vec::from_elem;

    let mut slots: HashMap<String, Rect> = HashMap::with_capacity(textures.len());
    let (width, height) = calculate_size(&textures);

    let mut texture = Texture {
        width,
        height,
        pixels: from_elem(0, (width * height) as usize), // TODO : Temporary, will be vec![0;...]
    };
    let mut free_rects = vec![Rect { width, height, x: 0, y: 0 }];
    for (name, tex) in textures {
        let w = tex.width;
        let h = tex.height;
        if let Some(rect) = place_rect(&mut free_rects, w, h) {
            for i in 0..w {
                for j in 0..h {
                    texture.set(rect.x + i, rect.y + j, tex.get(i, j));
                }
            }
            slots.insert(name, rect);
        }
    }
    return TextureAtlas { texture, slots };
}

// Very expensive function!
fn calculate_size(textures: &HashMap<String, &Texture>) -> (u32, u32) {
    let mut free_rects = vec![];
    let mut width = 256;
    let mut height = 256;

    loop {
        free_rects.push(Rect { width, height, x: 0, y: 0 });
        let mut found = true;
        for (_, tex) in textures {
            if let None = place_rect(&mut free_rects, tex.width, tex.height) {
                free_rects.clear();
                if width == height {
                    width *= 2;
                } else {
                    height *= 2;
                }
                found = false;
                break;
            }
        }

        if found { break; }
    }
    return (width, height);
}

fn place_rect(free: &mut Vec<Rect>, width: u32, height: u32) -> Option<Rect> {
    let rect = find_best_area(&free, width, height);
    if let Some(ref x) = rect {
        update_free_nodes(free, &x);
    }
    return rect;
}

fn update_free_nodes(free: &mut Vec<Rect>, rect: &Rect) {
    let mut size = free.len();
    let mut i = 0;
    while i < size {
        let free_rect = free[i].clone();
        if free_rect.intersects(rect) {
            free.remove(i);
            split_free_node(free, free_rect, rect);
            size -= 1;
        } else {
            i += 1;
        }
    }
    prune_free_list(free);
}

fn split_free_node(free: &mut Vec<Rect>, free_node: Rect, used: &Rect) {
    if used.x < free_node.x + free_node.width && used.x + used.width > free_node.x {
        // New node at the top side of the used node.
        if used.y > free_node.y && used.y < free_node.y + free_node.height {
            let mut new_node = free_node.clone();
            new_node.height = used.y - new_node.y;
            free.push(new_node);
        }

        // New node at the bottom side of the used node.
        if used.y + used.height < free_node.y + free_node.height {
            let mut new_node = free_node.clone();
            new_node.y = used.y + used.height;
            new_node.height = free_node.y + free_node.height - (used.y + used.height);
            free.push(new_node);
        }
    }
    if used.y < free_node.y + free_node.height && used.y + used.height > free_node.y {
        // New node at the left side of the used node.
        if used.x > free_node.x && used.x < free_node.x + free_node.width {
            let mut new_node = free_node.clone();
            new_node.width = used.x - new_node.x;
            free.push(new_node);
        }

        // New node at the right side of the used node.
        if used.x + used.width < free_node.x + free_node.width {
            let mut new_node = free_node.clone();
            new_node.x = used.x + used.width;
            new_node.width = free_node.x + free_node.width - (used.x + used.width);
            free.push(new_node);
        }
    }
}

fn prune_free_list(free: &mut Vec<Rect>) {
    let mut i = 0;
    while i < free.len() {
        let mut j = i + 1;
        while j < free.len() {
            if free[i].is_contained_in(&free[j]) {
                free.remove(i);
                i -= 1;
                break;
            }
            if free[j].is_contained_in(&free[i]) {
                free.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
}

fn find_best_area(free: &Vec<Rect>, width: u32, height: u32) -> Option<Rect> {
    let max_u32 = u32::max_value();

    let mut best_area_fit = max_u32;
    let mut best_short_side_fit = max_u32;

    let mut x = 0;
    let mut y = 0;
    for rect in free {
        if width <= rect.width && height <= rect.height {
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
    return if best_area_fit == max_u32 { None } else { Some(Rect { x, y, width, height }) };
}