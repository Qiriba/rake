use std::ptr;
use crate::{point, triangulate_ear_clipping, Point2D, Polygon2D};
use crate::texture::Texture;

#[derive(Clone)]
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,  // Farbwerte in 0xRRGGBBAA)
    pub z_buffer: Vec<f32>, // Tiefenwerte für jeden pixel, index identisch mit pixels
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

    ///Füllt den Framebuffer mit Schwarz und Z Wert von Unendlich
    pub(crate) fn clear(&mut self) {
        unsafe {
            let pixel_ptr = self.pixels.as_mut_ptr();
            for i in 0..self.pixels.len() {
                ptr::write(pixel_ptr.offset(i as isize), 0xFF000000);
            }
        }

        unsafe {
            let buffer_ptr = self.z_buffer.as_mut_ptr();
            for i in 0..self.pixels.len() {
                ptr::write(buffer_ptr.offset(i as isize), f32::INFINITY);
            }
        }
    }

    ///Erstellt Dreiecke aus dem gegebenen Polygon und rasteriziert diese in den Frambuffer, mit oder ohne Textur
    pub(crate) fn draw_polygon(&mut self, polygon: &Polygon2D, texture: Option<&Texture>, color: u32) {
        if let Some(texture) = texture {
            // Texturiertes Rendering
            let triangles = triangulate_ear_clipping(polygon);
            for (v0, v1, v2) in triangles {
                let uv0 = v0.1; // Textur-UV von Vertex 0
                let uv1 = v1.1; // Textur-UV von Vertex 1
                let uv2 = v2.1; // Textur-UV von Vertex 2

                self.rasterize_triangle_with_texture(v0.0, v1.0, v2.0, uv0, uv1, uv2, texture);
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
        // Snap vertices to pixel grid
        let v0 = point::snap_to_pixel(v0);
        let v1 = point::snap_to_pixel(v1);
        let v2 = point::snap_to_pixel(v2);

        // Compute bounding box, clamped to screen size
        let min_x = v0.x.min(v1.x).min(v2.x).max(0.0) as i32;
        let max_x = v0.x.max(v1.x).max(v2.x).min((self.width - 1) as f32) as i32;
        let min_y = v0.y.min(v1.y).min(v2.y).max(0.0) as i32;
        let max_y = v0.y.max(v1.y).max(v2.y).min((self.height - 1) as f32) as i32;

        // Edge function to calculate barycentric weights
        let edge_function = |a: &Point2D, b: &Point2D, p: &Point2D| -> i32 {
            ((p.x - a.x) as i32 * (b.y - a.y) as i32) - ((p.y - a.y) as i32 * (b.x - a.x) as i32)
        };

        // Compute the area of the triangle (for barycentric coordinate normalization)
        let triangle_area = edge_function(&v0, &v1, &v2) as f32;
        if triangle_area == 0.0 {
            return; // Degenerate triangle, no rendering required
        }
        let inv_area = 1.0 / triangle_area;

        // Loop over bounding box
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
                let w2 = 1.0 - w0 - w1; // Optional optimization: reuse previously computed weights

                // Check if the point is inside the triangle
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    // Interpolate depth
                    let z_interpolated = w0 * v0.z + w1 * v1.z + w2 * v2.z;

                    // Determine pixel index
                    let pixel_index = y as usize * self.width + x as usize;

                    // Depth test
                    if z_interpolated < self.z_buffer[pixel_index] {
                        self.z_buffer[pixel_index] = z_interpolated;
                        self.pixels[pixel_index] = color; // Set the pixel color
                    }
                }
            }
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

        // Textur-Größe
        let tex_width = texture.width as f32;
        let tex_height = texture.height as f32;

        // Berechnung der Bounding Box des Dreiecks
        let min_x = v0.x.min(v1.x).min(v2.x).max(0.0) as usize;
        let max_x = v0.x.max(v1.x).max(v2.x).min(self.width as f32) as usize;
        let min_y = v0.y.min(v1.y).min(v2.y).max(0.0) as usize;
        let max_y = v0.y.max(v1.y).max(v2.y).min(self.height as f32) as usize;
        // Pre-Berechnung der Determinante (Dreieckskalierung)
        let det = (v1.x - v0.x) * (v2.y - v0.y) - (v2.x - v0.x) * (v1.y - v0.y);

        if det.abs() < f32::EPSILON {
            return; // Degeneriertes Dreieck, kein Rendering nötig
        }


        // Schleife über alle Pixel innerhalb der Bounding Box
        for y in min_y..max_y {

            for x in min_x..max_x {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                // Baryzentrische Koordinaten berechnen
                let w0 = (v1.x - px) * (v2.y - py) - (v2.x - px) * (v1.y - py);
                let w1 = (v2.x - px) * (v0.y - py) - (v0.x - px) * (v2.y - py);
                let w2 = (v0.x - px) * (v1.y - py) - (v1.x - px) * (v0.y - py);
                // Prüfe, ob das Pixel innerhalb des Dreiecks liegt
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

                    // Berechne den Textur-Index und lese die Farbe
                    let tex_index = (tex_y * texture.width + tex_x) * 4;
                    let color = &texture.data[tex_index..tex_index + 4];

                    // Konvertiere RGBA (u8) in ein u32-Format
                    let color_u32 = rgba_to_u32([color[0], color[1], color[2], color[3]]);

                    // Berechne den Z-Index (falls Z-Buffer benötigt)
                    let z = b0 * v0.z + b1 * v1.z + b2 * v2.z;
                    let buffer_index = y * self.width + x;

                    // Z-Test: nur Pixel schreiben, die näher sind
                    if z < self.z_buffer[buffer_index] {
                        self.z_buffer[buffer_index] = z;
                        self.pixels[buffer_index] = color_u32;
                    }
                }
            }

        }
    }




}

fn rgba_to_u32(rgba: [u8; 4]) -> u32 {
    ((rgba[3] as u32) << 24) | // Alpha
        ((rgba[0] as u32) << 16) | // Rot
        ((rgba[1] as u32) << 8)  | // Grün
        (rgba[2] as u32)          // Blau
}
