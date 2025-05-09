use crate::Point2D;
use crate::texture::Texture;
use crate::{Matrix4x4, Point};
use std::sync::Arc;

#[derive(Debug)]
pub struct Polygon {
    pub vertices: Vec<Point>,
    pub(crate) tex_coords: Vec<(f32, f32)>,
    pub texture: Option<Arc<Texture>>,
    pub color: u32,
}

impl Polygon {
    pub fn new(colorout: u32) -> Self {
        Polygon {
            vertices: Vec::new(),
            tex_coords: Vec::new(),
            texture: None,
            color: colorout,
        }
    }

    pub fn set_texture(&mut self, texture: Arc<Texture>) {
        self.texture = Some(texture);
    }
    pub fn set_tex_coords(&mut self, tex_vec: Vec<(f32, f32)>) {
        self.tex_coords = tex_vec;
    }

    pub fn set_color(&mut self, color: u32) {
        self.color = color;
    }

    pub fn add_point(&mut self, point: Point) {
        self.vertices.push(point);
    }

    pub fn transform_full(
        &mut self,
        translation: (f32, f32, f32), // Verschiebung in (x, y, z)
        rotation: (f32, f32, f32),    // Rotation in (x, y, z) [in Radiant]
        scale: (f32, f32, f32),       // Skalierung in (x, y, z)
    ) {
        // Erzeuge die Transformationsmatrizen
        let translation_matrix = Matrix4x4::translate(translation.0, translation.1, translation.2);
        let rotation_x_matrix = Matrix4x4::rotate_x(rotation.0);
        let rotation_y_matrix = Matrix4x4::rotate_y(rotation.1);
        let rotation_z_matrix = Matrix4x4::rotate_z(rotation.2);
        let scaling_matrix = Matrix4x4::scale(scale.0, scale.1, scale.2);

        // Kombiniere alle Matrizen: Skalieren -> Rotieren (X -> Y -> Z) -> Verschieben
        let combined_matrix = translation_matrix
            .multiply(&rotation_z_matrix)
            .multiply(&rotation_y_matrix)
            .multiply(&rotation_x_matrix)
            .multiply(&scaling_matrix);

        // Wende die kombinierte Matrix auf jedes der Vertexe an
        for vertex in &mut self.vertices {
            *vertex = combined_matrix.multiply_point(vertex);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Polygon2D {
    pub vertices: Vec<Point2D>,
    pub uv_coords: Vec<(f32, f32)>,
}

pub fn project_polygon(
    polygon: &Polygon,
    view_matrix: &Matrix4x4,
    projection_matrix: &Matrix4x4,
    screen_width: usize,
    screen_height: usize,
) -> Polygon2D {
    let mut vertices_2d: Vec<Point2D> = Vec::new();
    let mut uv_coords_2d: Vec<(f32, f32)> = Vec::new();

    // Transformiere alle Punkte in den View-Space
    let mut view_vertices: Vec<Point> = polygon
        .vertices
        .iter()
        .map(|vertex| view_matrix.multiply_point(vertex))
        .collect();

    // Clippe gegen die Near-Plane damit nicht komische obstruktionen entstehen
    let near_plane = 0.1;
    view_vertices = clip_polygon_to_near_plane(&view_vertices, near_plane);

    // Prüfe ob das Polygon noch existiert
    if view_vertices.len() < 3 {
        return Polygon2D {
            vertices: vertices_2d,
            uv_coords: uv_coords_2d,
        };
    }

    // Projiziere alle übriggebliebenen Punkte
    for (vertex, uv) in view_vertices.iter().zip(&polygon.tex_coords) {
        // Projiziere den Punkt in den Clip-Space
        let projected = projection_matrix.multiply_point(vertex);

        // Perspektivische Division
        let x_ndc = projected.x / projected.z;
        let y_ndc = projected.y / projected.z;

        // Konvertiere in Bildschirmkoordinaten
        let screen_x = ((screen_width as f32 / 2.0) * (1.0 + x_ndc)).round();
        let screen_y = ((screen_height as f32 / 2.0) * (1.0 - y_ndc)).round();

        // Füge den Punkt in die 2DVertex-Liste ein
        vertices_2d.push(Point2D {
            x: screen_x,
            y: screen_y,
            z: projected.z, // Tiefeninformation ändern sich nicht
        });

        uv_coords_2d.push(*uv);
    }

    Polygon2D {
        vertices: vertices_2d,
        uv_coords: uv_coords_2d,
    }
}

fn clip_polygon_to_near_plane(vertices: &Vec<Point>, near: f32) -> Vec<Point> {
    let mut clipped_vertices = Vec::new();

    for i in 0..vertices.len() {
        let current = vertices[i];
        let next = vertices[(i + 1) % vertices.len()];

        let current_inside = current.z >= near;
        let next_inside = next.z >= near;

        // Beide Punkte innerhalb
        if current_inside && next_inside {
            clipped_vertices.push(next);
        }
        // Schnittpunkt
        else if current_inside || next_inside {
            let t = (near - current.z) / (next.z - current.z);
            let intersection = Point {
                x: current.x + t * (next.x - current.x),
                y: current.y + t * (next.y - current.y),
                z: near,
            };

            if current_inside {
                clipped_vertices.push(intersection);
            } else {
                clipped_vertices.push(intersection);
                clipped_vertices.push(next);
            }
        }
    }

    clipped_vertices
}
