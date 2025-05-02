use std::sync::Arc;
use crate::{Matrix4x4, Point};
use crate::Point2D;
use crate::texture::Texture;

#[derive(Debug)]
pub struct Polygon {
    pub vertices: Vec<Point>,
    pub(crate) tex_coords: Vec<(f32, f32)>, /// Texturkoordinaten für jeden Eckpunkt
    pub texture: Option<Arc<Texture>>,

    pub color: u32
}

impl Polygon {
    pub fn new(colorout: u32) -> Self {
        Polygon {
            vertices: Vec::new(),
            tex_coords: Vec::new(),
            texture: None,
            color: colorout
        }
    }

    pub fn set_texture(&mut self, texture: Arc<Texture>) {
        self.texture = Some(texture);
    }
    pub fn set_tex_coords(&mut self, vect: Vec<(f32, f32)>) {
        self.tex_coords = vect;
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
        rotation: (f32, f32, f32),   // Rotation in (x, y, z) [in Radiant]
        scale: (f32, f32, f32),      // Skalierung in (x, y, z)
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

        // Wende die kombinierte Matrix auf jedes Vertex an
        for vertex in &mut self.vertices {
            *vertex = combined_matrix.multiply_point(vertex);
        }
    }
    pub fn rotate_around_center(&mut self, rotation: (f32, f32, f32)) {
        // 1. Berechne den Mittelpunkt des Polygons
        let mut center_x = 0.0;
        let mut center_y = 0.0;
        let mut center_z = 0.0;
        let vertex_count = self.vertices.len() as f32;

        for vertex in &self.vertices {
            center_x += vertex.x;
            center_y += vertex.y;
            center_z += vertex.z;
        }

        center_x /= vertex_count;
        center_y /= vertex_count;
        center_z /= vertex_count;

        // 2. Verschiebe das Polygon relativ zum Ursprung
        let translation_to_origin = (-center_x, -center_y, -center_z);
        self.transform_full(translation_to_origin, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));

        // 3. Führe die Rotation um den Ursprung durch
        self.transform_full((0.0, 0.0, 0.0), rotation, (1.0, 1.0, 1.0));

        // 4. Verschiebe das Polygon zurück an seinen ursprünglichen Ort
        let translation_back = (center_x, center_y, center_z);
        self.transform_full(translation_back, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
    }
}

#[derive(Clone, Debug)]
pub struct Polygon2D {
    pub vertices: Vec<Point2D>,
    pub uv_coords: Vec<(f32, f32)>
}

