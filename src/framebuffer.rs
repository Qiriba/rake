use std::ptr;
use crate::{point, Point2D, Polygon2D};
use crate::texture::Texture;

#[derive(Clone)]
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,  // Farbwerte in 0xRRGGBBAA)
    pub z_buffer: Vec<f32>, // Tiefenwerte für jeden pixel, index gleich mit pixels
}

impl Framebuffer {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height], // Schwarzes Bild
            z_buffer: vec![f32::INFINITY; width * height], // Z-Buffer initial auf unendlich um andere werte drüber zu Zeichnen
        }
    }

    pub(crate) fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.pixels.resize(self.width * self.height, 0);
        self.z_buffer.resize(self.width * self.height, f32::INFINITY);
    }
    ///Füllt den Framebuffer mit Schwarz und Z Werte von Unendlich
    pub(crate) fn clear(&mut self) {
        unsafe {
            let pixel_ptr = self.pixels.as_mut_ptr();
            for i in 0..self.width * self.height {
                ptr::write(pixel_ptr.offset(i as isize), 0xFF000000);
            }

            let buffer_ptr = self.z_buffer.as_mut_ptr();
            for i in 0..self.width * self.height {
                ptr::write(buffer_ptr.offset(i as isize), f32::INFINITY);
            }
        }
    }

    ///Erstellt Dreiecke aus dem gegebenen Polygon und rasterisiert diese in den Frambuffer, mit oder ohne Textur
    pub(crate) fn draw_polygon(&mut self, polygon: &Polygon2D, texture: Option<&Texture>, color: u32) {
        if let Some(texture) = texture {
            // Texturiertes Rendering
            let triangles = triangulate_ear_clipping(polygon);
            for (v0, v1, v2) in triangles {
                self.rasterize_triangle_with_texture(v0.0, v1.0, v2.0, v0.1, v1.1, v2.1, texture);
            }
        } else {
            // Einfarbiges Rendering
            let triangles = triangulate_ear_clipping(polygon);
            for (v0, v1, v2) in triangles {
                self.rasterize_triangle(v0.0, v1.0, v2.0, color);
            }
        }
    }


    fn rasterize_triangle(&mut self, v0: Point2D, v1: Point2D, v2: Point2D, color: u32) {
        let v0 = point::snap_to_pixel(v0);
        let v1 = point::snap_to_pixel(v1);
        let v2 = point::snap_to_pixel(v2);

        // bounding box, auf screen size clamped
        let min_x = v0.x.min(v1.x).min(v2.x).max(0.0) as i32;
        let max_x = v0.x.max(v1.x).max(v2.x).min((self.width - 1) as f32) as i32;
        let min_y = v0.y.min(v1.y).min(v2.y).max(0.0) as i32;
        let max_y = v0.y.max(v1.y).max(v2.y).min((self.height - 1) as f32) as i32;


        // fläche des dreiecks berechnen (für barycentric coordinate normalization)
        let triangle_area = edge_function(&v0, &v1, &v2) as f32;
        if triangle_area == 0.0 {
            return; // kein rendering weil degeneriert
        }
        let inv_area = 1.0 / triangle_area;

        // nur bounding box loop
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = Point2D {
                    x: x as f32,
                    y: y as f32,
                    z: 0.0,
                };

                // Calculate barycentric coordinates
                let w0 = edge_function(&v1, &v2, &p) as f32 * inv_area;
                let w1 = edge_function(&v2, &v0, &p) as f32 * inv_area;
                let w2 = 1.0 - w0 - w1;

                // Check ob punkt im dreieck
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let z_interpolated = w0 * v0.z + w1 * v1.z + w2 * v2.z;

                    let pixel_index = y as usize * self.width + x as usize;

                    if z_interpolated < self.z_buffer[pixel_index] {
                        self.z_buffer[pixel_index] = z_interpolated;
                        self.pixels[pixel_index] = color;
                    }
                }
            }
        }

        // Edge function to calculate barycentric weights
        fn edge_function (a: &Point2D, b: &Point2D, p: &Point2D) -> i32 {
            ((p.x - a.x) as i32 * (b.y - a.y) as i32) - ((p.y - a.y) as i32 * (b.x - a.x) as i32)
        }
    }

    fn rasterize_triangle_with_texture(
        &mut self,
        v0: Point2D,
        v1: Point2D,
        v2: Point2D,
        uv0: (f32, f32),
        uv1: (f32, f32),
        uv2: (f32, f32),
        texture: &Texture,
    ) {

        let tex_width = texture.width as f32;
        let tex_height = texture.height as f32;

        // Berechnung der Bounding Box des Dreiecks
        let min_x = v0.x.min(v1.x).min(v2.x).max(0.0) as usize;
        let max_x = v0.x.max(v1.x).max(v2.x).min(self.width as f32) as usize;
        let min_y = v0.y.min(v1.y).min(v2.y).max(0.0) as usize;
        let max_y = v0.y.max(v1.y).max(v2.y).min(self.height as f32) as usize;
        // Vorberechnung der Determinante für Dreieckskalierung
        let det = (v1.x - v0.x) * (v2.y - v0.y) - (v2.x - v0.x) * (v1.y - v0.y);

        if det.abs() < f32::EPSILON {
            return; // Degeneriertes Dreieck
        }

        // Schleife über alle Pixel nur innerhalb der Bounding Box
        for y in min_y..max_y {
            for x in min_x..max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                // Baryzentrische Koordinaten berechnen
                let w0 = (v1.x - px) * (v2.y - py) - (v2.x - px) * (v1.y - py);
                let w1 = (v2.x - px) * (v0.y - py) - (v0.x - px) * (v2.y - py);
                let w2 = (v0.x - px) * (v1.y - py) - (v1.x - px) * (v0.y - py);

                // Prüfe ob der Pixel innerhalb des Dreiecks liegt
                if w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0 {
                    // Normiere die baryzentrischen Koordinaten
                    let denom = 1.0 / det;
                    let b0 = w0 * denom;
                    let b1 = w1 * denom;
                    let b2 = w2 * denom;

                    // Interpoliere die UV-Koordinaten
                    let u = b0 * uv0.0 + b1 * uv1.0 + b2 * uv2.0;
                    let v = b0 * uv0.1 + b1 * uv1.1 + b2 * uv2.1;

                    // Skalierung auf Texturgröße
                    let tex_x = (u * tex_width).clamp(0.0, tex_width - 1.0) as usize;
                    let tex_y = (v * tex_height).clamp(0.0, tex_height - 1.0) as usize;

                    // Berechne den Textur-Index
                    let tex_index = (tex_y * texture.width + tex_x) * 4;
                    let color = &texture.data[tex_index..tex_index + 4];

                    let color_u32 = rgba_to_u32([color[0], color[1], color[2], color[3]]);

                    let z = b0 * v0.z + b1 * v1.z + b2 * v2.z;
                    let buffer_index = y * self.width + x;

                    // nur Pixel schreiben, die näher sind
                    if z < self.z_buffer[buffer_index] {
                        self.z_buffer[buffer_index] = z;
                        self.pixels[buffer_index] = color_u32;
                    }
                }
            }
        }
    }
}


#[inline(always)]
fn rgba_to_u32(rgba: [u8; 4]) -> u32 {
        ((rgba[3] as u32) << 24) | // Alpha
        ((rgba[0] as u32) << 16) | // Rot
        ((rgba[1] as u32) << 8)  | // Grün
        (rgba[2] as u32)          // Blau
}

fn triangulate_ear_clipping(
    polygon: &Polygon2D,
) -> Vec<((Point2D, (f32, f32)), (Point2D, (f32, f32)), (Point2D, (f32, f32)))> {
    let mut vertices = polygon.vertices.clone(); // Kopiere die Punkte des Polygons
    let mut uv_coords = polygon.uv_coords.clone(); // Kopiere die UV-Koordinaten des Polygons
    let mut triangles = Vec::new();
    // Rechteck/Quadrat: Sonderfall – einfache Zwei-Dreiecks-Zerlegung
    if vertices.len() == 4 {
        return vec![
            (
                (vertices[0], uv_coords[0]),
                (vertices[1], uv_coords[1]),
                (vertices[2], uv_coords[2]),
            ),
            (
                (vertices[2], uv_coords[2]),
                (vertices[3], uv_coords[3]),
                (vertices[0], uv_coords[0]),
            ),
        ];
    }

    ensure_ccw(&mut vertices);

    // Starte die Triangulation
    while vertices.len() > 3 {
        let mut ear_found = false;

        // Finde ein "Ohr" im Polygon
        for i in 0..vertices.len() {
            // Vorheriger, aktueller und nächster Punkt
            let prev = vertices[(i + vertices.len() - 1) % vertices.len()];
            let prev_uv = uv_coords[(i + uv_coords.len() - 1) % uv_coords.len()];
            let curr = vertices[i];
            let curr_uv = uv_coords[i];
            let next = vertices[(i + 1) % vertices.len()];
            let next_uv = uv_coords[(i + 1) % uv_coords.len()];

            // Prüfe, ob ein Ohr gefunden wurde
            if is_ear(prev, curr, next, &vertices) {
                // Füge das Ohr als ein Dreieck hinzu
                triangles.push((
                    (prev, prev_uv),
                    (curr, curr_uv),
                    (next, next_uv),
                ));

                // Entferne den aktuellen Punkt und seine UVs aus der Liste
                vertices.remove(i);
                uv_coords.remove(i);

                ear_found = true;
                break;
            }
        }

        // Wenn nach einem Durchlauf kein Ohr gefunden wurde, ist das Polygon wahrscheinlich
        // ungültig oder zu komplex.
        if !ear_found {
            panic!("Triangulation fehlgeschlagen: Ungültiges oder zu komplexes Polygon!");
        }
    }

    // Füge das letzte verbleibende Dreieck hinzu (wenn noch genau 3 Punkte übrig sind)
    if vertices.len() == 3 {
        triangles.push((
            (vertices[0], uv_coords[0]),
            (vertices[1], uv_coords[1]),
            (vertices[2], uv_coords[2]),
        ));
    }

    triangles
}

#[inline(always)]
fn is_ear(prev: Point2D, curr: Point2D, next: Point2D, vertices: &[Point2D]) -> bool {
    if !is_ccw(prev, curr, next) {
        return false; // Das Dreieck ist nicht gegen den Uhrzeigersinn
    }

    // Prüfe, ob ein anderer Punkt innerhalb des Dreiecks liegt
    for &v in vertices {
        if v != prev && v != curr && v != next && is_point_in_triangle(v, prev, curr, next) {
            return false;
        }
    }
    true
}

#[inline(always)]
fn is_ccw(p1: Point2D, p2: Point2D, p3: Point2D) -> bool {

    let cross_product = (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x);

    if cross_product > 0.0 {
        true // Gegen den Uhrzeigersinn
    } else {
        false
    }
}

#[inline(always)]
fn is_polygon_ccw(vertices: &[Point2D]) -> bool {
    let mut sum = 0.0;
    for i in 0..vertices.len() {
        let current = vertices[i];
        let next = vertices[(i + 1) % vertices.len()];
        sum += (next.x - current.x) * (next.y + current.y);
    }
    if sum > 0.0 {
        true // Polygon in Gegen-Uhrzeigersinn
    } else {
        false
    }
}

#[inline(always)]
fn ensure_ccw(vertices: &mut Vec<Point2D>) {
    if !is_polygon_ccw(vertices) {
        vertices.reverse();
    }
}

#[inline(always)]
fn is_point_in_triangle(p: Point2D, a: Point2D, b: Point2D, c: Point2D) -> bool {
    let det = |p1: Point2D, p2: Point2D, p3: Point2D| -> f32 {
        (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x)
    };

    let d1 = det(p, a, b);
    let d2 = det(p, b, c);
    let d3 = det(p, c, a);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos) || (d1.abs() < f32::EPSILON || d2.abs() < f32::EPSILON || d3.abs() < f32::EPSILON)
}