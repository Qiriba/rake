use std::env::vars;
use crate::{Point, Polygon};

pub fn load_obj(path: &str) -> Result<Vec<Polygon>, String> {
    // Read the file to string
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read OBJ file: {}", e))?;

    let mut vertices = Vec::new();
    let mut tex_coords = Vec::new();
    let mut normals = Vec::new();
    let mut polygons = Vec::new();

    // Default color - you might want to use a parameter later
    let default_color = 0xFFFFFFFF;

    // Parse the file line by line
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue; // Skip comments and empty lines
        }

        let mut parts = line.split_whitespace();
        let identifier = parts.next().unwrap_or("");

        match identifier {
            "v" => {
                // Parse vertex position (v x y z)
                let x = parts
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                let y = parts
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                let z = parts
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                if vars().any(|(k, _)| k == "Z_UP") {
                    // Defines z as up direction
                    vertices.push(Point::new(x, y, z));
                } else {
                    // Defines y as up direction
                    vertices.push(Point::new(x, -z, y));
                }
            }
            "vt" => {
                // Parse texture coordinates (vt u v)
                let u = parts
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                let v = parts
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                tex_coords.push((u, v));
            }
            "vn" => {
                // Parse normals (vn x y z)
                let nx = parts
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                let ny = parts
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                let nz = parts
                    .next()
                    .and_then(|s| s.parse::<f32>().ok())
                    .unwrap_or(0.0);
                normals.push(Point::new(nx, ny, nz));
            }
            "f" => {
                // Parse face (f v1/vt1/vn1 v2/vt2/vn2 v3/vt3/vn3 ...)
                let mut face_vertices = Vec::new();
                let mut face_tex_coords = Vec::new();

                for vertex_str in parts {
                    let indices: Vec<&str> = vertex_str.split('/').collect();

                    if indices.is_empty() {
                        continue;
                    }

                    // Parse vertex index (1-based in OBJ format)
                    if let Some(v_idx_str) = indices.get(0) {
                        if let Ok(v_idx) = v_idx_str.parse::<usize>() {
                            // OBJ indices start at 1, so subtract 1
                            if v_idx > 0 && v_idx <= vertices.len() {
                                face_vertices.push(vertices[v_idx - 1]);

                                // Parse texture coordinate index if present
                                if indices.len() > 1 && !indices[1].is_empty() {
                                    if let Ok(vt_idx) = indices[1].parse::<usize>() {
                                        if vt_idx > 0 && vt_idx <= tex_coords.len() {
                                            face_tex_coords.push(tex_coords[vt_idx - 1]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Create polygon from the face if we have at least 3 vertices
                if face_vertices.len() >= 3 {
                    let mut polygon = Polygon::new(default_color);

                    // Add vertices to polygon
                    for vertex in face_vertices {
                        polygon.add_point(vertex);
                    }

                    // Add texture coordinates if available
                    if !face_tex_coords.is_empty() {
                        face_tex_coords.swap(1, 2);
                        polygon.tex_coords = face_tex_coords;
                    }

                    polygons.push(polygon);
                }
            }
            _ => {} // Ignore other lines
        }
    }

    // Return the loaded polygons
    if polygons.is_empty() {
        Err("No valid polygons found in the OBJ file".to_string())
    } else {
        println!(
            "Loaded {} vertices and {} polygons from {}",
            vertices.len(),
            polygons.len(),
            path
        );
        Ok(polygons)
    }
}

pub fn load_test_cube() -> Vec<Polygon> {
    // Create a simple cube
    let mut polygons = Vec::new();

    // Front face (red)
    let mut front = Polygon::new(0xFF0000FF);
    front.add_point(Point::new(-1.0, -1.0, -2.0));
    front.add_point(Point::new(1.0, -1.0, -2.0));
    front.add_point(Point::new(1.0, 1.0, -2.0));
    front.add_point(Point::new(-1.0, 1.0, -2.0));

    // Back face (green)
    let mut back = Polygon::new(0x00FF00FF);
    back.add_point(Point::new(-1.0, -1.0, -4.0));
    back.add_point(Point::new(-1.0, 1.0, -4.0));
    back.add_point(Point::new(1.0, 1.0, -4.0));
    back.add_point(Point::new(1.0, -1.0, -4.0));

    // Right face (blue)
    let mut right = Polygon::new(0x0000FFFF);
    right.add_point(Point::new(1.0, -1.0, -4.0));
    right.add_point(Point::new(1.0, 1.0, -4.0));
    right.add_point(Point::new(1.0, 1.0, -2.0));
    right.add_point(Point::new(1.0, -1.0, -2.0));

    // Left face (yellow)
    let mut left = Polygon::new(0xFFFF00FF);
    left.add_point(Point::new(-1.0, -1.0, -4.0));
    left.add_point(Point::new(-1.0, -1.0, -2.0));
    left.add_point(Point::new(-1.0, 1.0, -2.0));
    left.add_point(Point::new(-1.0, 1.0, -4.0));

    // Top face (cyan)
    let mut top = Polygon::new(0x00FFFFFF);
    top.add_point(Point::new(-1.0, 1.0, -4.0));
    top.add_point(Point::new(-1.0, 1.0, -2.0));
    top.add_point(Point::new(1.0, 1.0, -2.0));
    top.add_point(Point::new(1.0, 1.0, -4.0));

    // Bottom face (magenta)
    let mut bottom = Polygon::new(0xFF00FFFF);
    bottom.add_point(Point::new(-1.0, -1.0, -4.0));
    bottom.add_point(Point::new(1.0, -1.0, -4.0));
    bottom.add_point(Point::new(1.0, -1.0, -2.0));
    bottom.add_point(Point::new(-1.0, -1.0, -2.0));

    // Add the polygons to the vector
    polygons.push(front);
    polygons.push(back);
    polygons.push(right);
    polygons.push(left);
    polygons.push(top);
    polygons.push(bottom);

    // Return the created polygons
    polygons
}