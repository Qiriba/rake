use std::ptr;

#[derive(Clone)]
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,  // Store the color values (e.g., 0xRRGGBBAA)
    pub z_buffer: Vec<f32>, // Store depth values for each pixel
}

impl Framebuffer {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height], // Schwarzes Bild
            z_buffer: vec![f32::INFINITY; width * height], // Z-Buffer initial auf "unendlich"
        }
    }

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

    pub(crate) fn draw_polygon(&mut self, polygon: &Polygon2D, color: u32) {

        // Kein gültiges Polygon wenn weniger als 3 Vertexe
        if polygon.vertices.len() < 3 {
            return;
        }

        //Erstellt dreiecke aus dem Polygon
        let triangles = triangulate_ear_clipping(polygon);

        // Render jedes der Dreiecke separat
        for (v0, v1, v2) in triangles {
            self.rasterize_triangle(v0, v1, v2, color);
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

        // Compute edge functions (integer arithmetic)
        let edge_function = |a: &Point2D, b: &Point2D, p: &Point2D| -> i32 {
            ((p.x - a.x) as i32 * (b.y - a.y) as i32) - ((p.y - a.y) as i32 * (b.x - a.x) as i32)
        };

        // Compute triangle area (denominator for barycentric coordinates)
        let area = edge_function(&v0, &v1, &v2) as f32;
        if area == 0.0 {
            return; // Degenerate triangle, skip rendering
        }
        let inv_area = 1.0 / area;

        // Iterate over the bounding box (scanline-based)
        for y in min_y..=max_y {
            let mut inside_found = false;
            let mut x_start = min_x;
            let mut x_end = max_x;

            // Optimize X start and end (reduce unnecessary iterations)
            while x_start < max_x && edge_function(&v1, &v2, &Point2D { x: x_start as f32, y: y as f32, z: 0.0 }) < 0 {
                x_start += 1;
            }
            while x_end > min_x && edge_function(&v0, &v1, &Point2D { x: x_end as f32, y: y as f32, z: 0.0 }) < 0 {
                x_end -= 1;
            }

            for x in x_start..=x_end {
                let p = Point2D { x: x as f32, y: y as f32, z: 0.0 };

                // Compute barycentric coordinates
                let w0 = edge_function(&v1, &v2, &p) as f32 * inv_area;
                let w1 = edge_function(&v2, &v0, &p) as f32 * inv_area;
                let w2 = 1.0 - w0 - w1; // Avoid redundant calculation

                // If all weights are inside [0,1], the pixel is inside the triangle
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    inside_found = true;

                    // Interpolate depth
                    let z = w0 * v0.z + w1 * v1.z + w2 * v2.z;
                    let index = y as usize * self.width + x as usize;

                    // Depth test
                    if z < self.z_buffer[index] {
                        self.z_buffer[index] = z;
                        self.pixels[index] = color;
                    }
                } else if inside_found {
                    // If we were inside and now we are not, break early
                    break;
                }
            }
        }
    }

}