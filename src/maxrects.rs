use texture::Rect;

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

pub fn place_rect(free: &mut Vec<Rect>, width: u32, height: u32) -> Option<Rect> {
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