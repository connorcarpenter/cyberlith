use std::collections::HashSet;

use log::{info, warn};
use ttf2mesh::{Quality, TTFFile, Value};

use asset_id::AssetId;
use asset_serde::json::{IconFileFrame, IconJson};

use crate::CliError;

pub fn convert_ttf_to_icon(ttf_file_name: &str) -> Result<(), CliError> {
    info!("Converting ttf to icon. input: {:?}", ttf_file_name);
    let mut ttf = TTFFile::from_file(ttf_file_name).unwrap();
    info!("ttf glyph count: {}", ttf.glyph_count());

    let mut output_file = IconJson::new();
    let palette_asset_id = AssetId::from_str("8273wa").unwrap();
    output_file.set_palette_asset_id(&palette_asset_id);

    for ascii_code in 32..=126 {
        // should be 32..=126
        let character = ascii_code as u8 as char;
        info!("ASCII Code: {}  Character: {}", ascii_code, character);

        let mut new_frame = IconFileFrame::new();

        let mut glyph = ttf.glyph_from_char(character).unwrap();
        let mesh_2d = match glyph.to_2d_mesh(Quality::Custom(1)) {
            Ok(mesh) => mesh,
            Err(e) => {
                warn!("Error: {:?}", e);
                output_file.add_frame(new_frame);
                continue;
            }
        };

        // find the bounding box of the glyph
        let mut max_vert_x = -10000.0;
        let mut min_vert_x = 10000.0;
        for vertex in mesh_2d.iter_vertices() {
            let (x, y) = vertex.val();
            if x > max_vert_x {
                max_vert_x = x;
            }
            if x < min_vert_x {
                min_vert_x = x;
            }
        }
        let mid_x = (max_vert_x + min_vert_x) / 2.0;

        // process and add vertices
        for vertex in mesh_2d.iter_vertices() {
            let (mut x, mut y) = vertex.val();

            x -= mid_x;
            x *= 150.0;
            y *= 150.0;
            y *= -1.0;
            y += 55.0;

            new_frame.add_vertex(x as i16, y as i16);
        }

        // add edges (derive from faces)
        let mut edges = HashSet::new();
        for face in mesh_2d.iter_faces() {
            let (v1, v2, v3) = face.val();
            let v1 = v1 as u16;
            let v2 = v2 as u16;
            let v3 = v3 as u16;

            // add edges
            insert_edge(&mut edges, v1, v2);
            insert_edge(&mut edges, v2, v3);
            insert_edge(&mut edges, v3, v1);
        }

        for (vert_a, vert_b) in edges {
            new_frame.add_edge(vert_a, vert_b);
        }

        // add faces
        let mut face_index = 0;
        for face in mesh_2d.iter_faces() {
            let (v1, v2, v3) = face.val();
            let v1 = v1 as u16;
            let v2 = v2 as u16;
            let v3 = v3 as u16;
            new_frame.add_face(face_index, 0, v1, v2, v3);
            face_index += 1;
        }

        output_file.add_frame(new_frame);
    }

    //info!("max_vert_x: {}  max_vert_y: {}  min_vert_x: {}  min_vert_y: {}", max_vert_x, max_vert_y, min_vert_x, min_vert_y);
    //std::process::exit(0);
    // write out icon file
    let output_bytes = output_file.write(&AssetId::gen_random());

    // remove .ttf from ttf_file_name
    let mut file_name_path = std::path::Path::new(ttf_file_name);
    let file_name = file_name_path.file_stem().unwrap().to_str().unwrap();
    let output_file_name = format!("{}.icon.json", file_name);

    info!(
        "Writing icon file: {:?}. (bytes: {:?})",
        output_file_name,
        output_bytes.len()
    );
    std::fs::write(output_file_name, output_bytes).unwrap();

    Ok(())
}

fn insert_edge(edges: &mut HashSet<(u16, u16)>, v1: u16, v2: u16) {
    if v1 < v2 {
        edges.insert((v1, v2));
    } else {
        edges.insert((v2, v1));
    }
}
