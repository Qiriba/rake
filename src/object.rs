use std::fs::File;
use std::io::{BufRead, BufReader};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use crate::point::Point;
use crate::Polygon;
use rayon::iter::IndexedParallelIterator;

pub fn parse_obj_file(file_path: &str) -> Result<(Vec<Point>, Vec<(Vec<usize>, Vec<usize>)>, Vec<(f32, f32)>), String> {
    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut tex_coords = Vec::new();
    let mut faces = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?.trim().to_string();

        if line.starts_with("v ") {
            let coords: Vec<f32> = line[2..]
                .split_whitespace()
                .filter_map(|part| part.parse::<f32>().ok())
                .collect();
            if coords.len() == 3 {
                vertices.push(Point::new(coords[0], -coords[2], coords[1]));
            }
        } else if line.starts_with("vt ") {
            let coords: Vec<f32> = line[3..]
                .split_whitespace()
                .filter_map(|part| part.parse::<f32>().ok())
                .collect();
            if coords.len() >= 2 {
                tex_coords.push((coords[0], coords[1]));
            }
        } else if line.starts_with("f ") {
            let mut v_indices = Vec::new();
            let mut t_indices = Vec::new();

            for part in line[2..].split_whitespace() {
                let mut split = part.split('/');
                let v_idx = split.next().and_then(|s| s.parse::<usize>().ok()).map(|i| i - 1);
                let t_idx = split.next().and_then(|s| s.parse::<usize>().ok()).map(|i| i - 1);

                if let (Some(v), Some(t)) = (v_idx, t_idx) {
                    v_indices.push(v);
                    t_indices.push(t);
                }
            }

            if v_indices.len() == 3 && t_indices.len() == 3 {
                let mut t_indices_fixed = t_indices.clone();
                t_indices_fixed.swap(1, 2);
                faces.push((v_indices, t_indices_fixed));
            } else if v_indices.len() >= 3 && t_indices.len() == v_indices.len() {
                faces.push((v_indices, t_indices));
            }
        }
    }

    Ok((vertices, faces, tex_coords))
}

pub fn process_faces(
    vertices: &Vec<Point>,
    faces: &Vec<(Vec<usize>, Vec<usize>)>,
    tex_coords: &Vec<(f32, f32)>
) -> Vec<Polygon> {
    faces
        .par_iter()
        .filter_map(|(v_indices, t_indices)| {
            if v_indices.len() < 3 {
                return None;
            }

            let points: Vec<Point> = v_indices.iter().map(|&i| vertices[i]).collect();
            let texs: Vec<(f32, f32)> = t_indices.iter().map(|&i| tex_coords[i]).collect();

            let mut polygon = Polygon::new(0xFFFFFFFF);
            for point in points {
                polygon.add_point(point);
            }
            polygon.set_tex_coords(texs);

            Some(polygon)
        })
        .collect()
}
