use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::Point2D;
use crate::point::Point; // Importiere deine Punktstruktur

pub fn parse_obj_file(file_path: &str) -> Result<(Vec<Point>, Vec<Vec<usize>>, ), String> {
    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);

    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    let mut uv:Vec<Vec<(f32, f32)>> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Failed to read line: {}", e))?.trim().to_string();
        if line.starts_with("v ") {
            // Parsen von Vertex-Daten
            let coords: Vec<f32> = line[2..]
                .split_whitespace()
                .filter_map(|part| part.parse::<f32>().ok())
                .collect();
            if coords.len() == 3 {
                vertices.push(Point::new(coords[0], coords[1], coords[2]));
            }
        } else if line.starts_with("f ") {
            // Parsen von Face-Daten
            let indices: Vec<usize> = line[2..]
                .split_whitespace()
                .filter_map(|part| part.split('/').next()?.parse::<usize>().ok())
                .map(|index| index - 1) // .obj beginnt bei 1, nicht bei 0
                .collect();
            if indices.len() >= 3 {
                faces.push(indices);
            }
        }
    }

    Ok((vertices, faces))
}