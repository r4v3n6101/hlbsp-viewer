extern crate image;

pub mod hl;

#[cfg(test)]
mod tests {
    #[test]
    fn list_textures_test() {
        use hl::{read_struct, texture::*, wad::*};
        use std::{fs::File, io::*};

        let name = "halflife.wad";
        let mut f: File = File::open(name).unwrap();
        let size = f.metadata().unwrap().len() as usize;

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        f.read_to_end(&mut buf).unwrap();

        ///////////////////////////////////////////////////////////////////////////////////////////
        let entries: Vec<DirEntry> = entries(&buf);
        entries.into_iter().for_each(|entry| {
            let tex_offset: usize = entry.file_pos as usize;
            let tex: MipTex = read_struct(&buf[tex_offset..]);
            let col_table = tex.get_color_table(&buf[tex_offset..]);
            let color: Vec<u8> = tex.read_texture(&buf[tex_offset..], col_table, 0);
            let out_name = format!("./images/{}.png", String::from_utf8_lossy(&tex.name)).replace('\0', "");

            let imgbuf = ::image::ImageBuffer::from_vec(tex.width, tex.height, color).unwrap();

            let mut fout = File::create(out_name).unwrap();
            ::image::ImageRgb8(imgbuf).save(&mut fout, ::image::PNG).unwrap();
        });
    }

    #[test]
    fn convert_test() {
        // TODO : Rewrite this into separate program
        // TODO : Buffered writer
        use std::{fs::File, io::*};
        use hl::{read_struct, bsp::*};

        let name = "gasworks.bsp";
        let mut f: File = File::open(name).unwrap();
        let size = f.metadata().unwrap().len() as usize;

        let mut buf: Vec<u8> = Vec::with_capacity(size);
        f.read_to_end(&mut buf).unwrap();

        let mut out = File::create("gasworks.obj").unwrap();

        let header: Header = read_struct(&buf);

        let vertices: Vertices = header.lumps[LUMP_VERTICES].read_array(&buf);
        let faces: Vec<Face> = header.lumps[LUMP_FACES].read_array(&buf);
        let surfedges: Vec<i32> = header.lumps[LUMP_SURFEDGES].read_array(&buf);
        let edges: Vec<Edge> = header.lumps[LUMP_EDGES].read_array(&buf);
        vertices.into_iter().for_each(|vertex| {
            out.write(format!("v {} {} {}\n", vertex.0, vertex.1, vertex.2).as_bytes()).unwrap();
        });
        out.write("\n\n\n".as_bytes()).unwrap();
        faces.into_iter().for_each(|face| {
            let mut face_str = String::new();
            face_str += "f";
            for i in 0..face.edges {
                let surfedge_i: u32 = face.first_edge + (i as u32);
                let surfedge: i32 = surfedges[surfedge_i as usize];
                let vert: u16;
                if surfedge > 0 {
                    vert = edges[surfedge as usize].vertices[0];
                } else {
                    vert = edges[-surfedge as usize].vertices[1];
                }
                face_str += &format!(" {}", vert + 1); // OBJ uses 1 as first index instead of 0
            }
            face_str += "\n";
            out.write(face_str.as_bytes()).unwrap();
        });
    }
}
