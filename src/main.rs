mod point;
pub use point::Point;
pub use point::Point2D;

mod polygon;
pub use polygon::Polygon;
pub use polygon::Polygon2D;

mod camera;
pub use camera::Camera;

mod framebuffer;
mod matrix4x4;
mod object;
mod texture;

pub use framebuffer::Framebuffer;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::texture::Texture;
use lazy_static::lazy_static;

use crate::matrix4x4::Matrix4x4;
use rayon::prelude::*;

const WINDOW_WIDTH: f64 = 800.0;
const WINDOW_HEIGHT: f64 = 600.0;
pub const TARGET_FPS: f32 = 60.0;

lazy_static! {
    static ref KEYS: Mutex<[bool; 256]> = Mutex::new([false; 256]);
}

fn render_scene(
    camera: &Camera,
    polygons: &Vec<Polygon>,
    framebuffer: &mut Framebuffer,
    width: u32,
    height: u32,
) {
    let view_matrix = camera.view_matrix(); // Recalculating view matrix after camera movement
    let projection_matrix = camera.projection_matrix();

    framebuffer.clear(); // Empty frame buffer so nothing gets overdrawn

    add_directional_gradient(&camera, &polygons, framebuffer);

    let projected_polygons: Vec<_> = polygons
        .par_iter()
        .filter_map(|polygon| {
            if is_backface(polygon, camera.position) {
                return None;
            }

            let projected = project_polygon(
                polygon,
                &view_matrix,
                &projection_matrix,
                width as usize,
                height as usize,
            );

            // Passes Option<&Texture>, if exists
            let texture_option = polygon.texture.as_ref();

            Some((projected, texture_option, polygon.color))
        })
        .collect();

    // Rendering frame buffer
    for (projected, texture, color) in projected_polygons {
        framebuffer.draw_polygon(&projected, texture, color);
    }
}

fn is_backface(polygon: &Polygon, camera_position: Point) -> bool {
    if polygon.vertices.len() < 3 {
        return true; // Can't be a valid polygon with less than 3 vertices
    }

    let edge1 = polygon.vertices[1] - polygon.vertices[0];
    let edge2 = polygon.vertices[2] - polygon.vertices[0];
    let normal = edge1.cross(edge2).normalize();

    let view_direction = (camera_position - polygon.vertices[0]).normalize();
    normal.dot(view_direction) < 0.0
}

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Point as SDLPoint;

const INITIAL_WIDTH: u32 = 320;
const INITIAL_HEIGHT: u32 = 240;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut camera = Camera::new(
        Point::new(5.0, 0.0, 0.0),                  // Starting position
        Point::new(0.0, 0.0, 1.0),                  // View direction
        Point::new(0.0, 1.0, 0.0),                  // "Up"-vector
        60.0,                                       // Field of View (FOV)
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32, // Seitenverhältnis
        0.1,                                        // Near clipping
        100.0,                                      // Far clipping
    );

    let window = video_subsystem
        .window("rake", INITIAL_WIDTH, INITIAL_HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    let frame_duration = Duration::from_secs_f32(1.0 / TARGET_FPS);
    let mut last_update = Instant::now();

    let mut width = INITIAL_WIDTH;
    let mut height = INITIAL_HEIGHT;

    let mut framebuffer = Framebuffer::new(width as usize, height as usize);

    let mut polygons = object::load_obj("./capsule.obj").unwrap_or_else(|e| {
        println!("Error loading OBJ file: {}", e);
        println!("Falling back to default cube");
        object::load_test_cube()
    });

    let texture = Texture::from_file("./capsule0.jpg");

    // Sharing texture for ressource management
    let shared_texture = Arc::new(texture);

    // Assign texture to polygons
    for polygon in &mut polygons {
        polygon.texture = Some(shared_texture.clone())
    }

    normalize_model(&mut polygons, 2.0);

    // let mut bbox_polygons: Vec<Polygon> = Vec::new();

    let mut mouse_captured = false;
    sdl_context.mouse().set_relative_mouse_mode(false);

    let mut show_texture = true;
    let mut skip_backfaces = true;
    // let mut show_bbox = false;

    println!("Starting SDL2 render loop");

    'running: loop {
        // Mouse movement tracking
        let mut mouse_delta = (0.0f32, 0.0f32);

        camera.update_forward();

        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Window { win_event, .. } => {
                    if let sdl2::event::WindowEvent::Resized(w, h) = win_event {
                        width = w as u32;
                        height = h as u32;
                        camera.update_ratio(width as f32 / height as f32);
                        framebuffer.resize(width as usize, height as usize);
                        println!("Window resized to: {}x{}", width, height);
                    }
                }
                Event::MouseMotion { xrel, yrel, .. } => {
                    if mouse_captured {
                        mouse_delta.0 = xrel as f32 * 2.5;
                        mouse_delta.1 = yrel as f32 * 2.5;
                    }
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    // Update key state
                    let mut keys = KEYS.lock().unwrap();
                    match key {
                        Keycode::W => keys['W' as usize] = true,
                        Keycode::A => keys['A' as usize] = true,
                        Keycode::S => keys['S' as usize] = true,
                        Keycode::D => keys['D' as usize] = true,
                        Keycode::T => {
                            show_texture = !show_texture;
                            println!("Show texture: {}", show_texture);
                        }
                        Keycode::Comma => focus_camera_on_model(&mut camera, &polygons),
                        Keycode::Space => keys['V' as usize] = true,
                        Keycode::B => {
                            skip_backfaces = !skip_backfaces;
                            println!("Skip backfaces: {}", skip_backfaces);
                        }
                        /*
                        Keycode::P => {
                            show_bbox = !show_bbox;
                            if show_bbox {
                                bbox_polygons = visualize_bounding_box(&polygons);
                                println!("Showing bounding box");
                            } else {
                                println!("Hiding bounding box");
                            }
                        }*/
                        Keycode::Tab => {
                            mouse_captured = !mouse_captured;
                            sdl_context.mouse().set_relative_mouse_mode(mouse_captured);
                            if mouse_captured {
                                mouse_delta = (0.0f32, 0.0f32);
                            }
                        }
                        _ => {}
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    // Update key state
                    let mut keys = KEYS.lock().unwrap();
                    match key {
                        Keycode::W => keys['W' as usize] = false,
                        Keycode::A => keys['A' as usize] = false,
                        Keycode::S => keys['S' as usize] = false,
                        Keycode::D => keys['D' as usize] = false,
                        Keycode::Space => keys['V' as usize] = false,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Limit frame rate
        let now = Instant::now();
        let elapsed = now.duration_since(last_update);
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
            continue;
        }

        // Calculate delta time
        let delta_time = now.duration_since(last_update).as_secs_f32();
        last_update = Instant::now();

        // Update camera (with mouse delta)
        {
            let keys = KEYS.lock().unwrap();
            camera.update_movement(delta_time, &*keys, mouse_delta);
        }

        // Render the scene
        if show_texture {
            render_scene(&camera, &polygons, &mut framebuffer, width, height);
            fb_to_canvas(&framebuffer, &mut canvas)
                .expect("Error converting framebuffer to canvas");
        } else {
            render_scene_sdl2(
                &camera,
                &polygons,
                &mut canvas,
                width,
                height,
                skip_backfaces,
            )?;
        }

        /*if show_bbox {
        } else {
            render_scene_sdl2(&polygons, &mut canvas, width, height, skip_backfaces)?;
        }*/

        canvas.present();
    }

    Ok(())
}

fn normalize_model(polygons: &mut Vec<Polygon>, target_size: f32) {
    // Find min/max bounds
    let mut min = Point::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Point::new(f32::MIN, f32::MIN, f32::MIN);

    for polygon in polygons.iter() {
        for vertex in &polygon.vertices {
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);

            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }
    }

    // Calculate center and size
    let center = Point::new(
        (min.x + max.x) / 2.0,
        (min.y + max.y) / 2.0,
        (min.z + max.z) / 2.0,
    );

    let size = (max.x - min.x).max((max.y - min.y).max(max.z - min.z));
    let scale = if size > 0.0 { target_size / size } else { 1.0 };

    // Normalize each vertex
    for polygon in polygons.iter_mut() {
        for vertex in &mut polygon.vertices {
            // Center the model
            *vertex = Point::new(
                vertex.x - center.x,
                vertex.y - center.y,
                vertex.z - center.z,
            );

            // Scale it to target size
            *vertex = Point::new(vertex.x * scale, vertex.y * scale, vertex.z * scale);
        }
    }

    println!("Model normalized: center={:?}, scale={}", center, scale);
}

fn fb_to_canvas(
    framebuffer: &Framebuffer,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
) -> Result<(), String> {
    let width = framebuffer.width;
    let height = framebuffer.height;

    // Create a texture from the framebuffer data
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::ARGB8888, width as u32, height as u32)
        .map_err(|e| e.to_string())?;

    // Update the texture with the framebuffer data
    texture
        .update(
            None,
            &vec32_to_u8array(&framebuffer.pixels),
            framebuffer.width as usize * 4,
        )
        .map_err(|e| e.to_string())?;

    // Clear the canvas
    canvas.clear();

    // Copy the texture to the canvas
    canvas.copy(&texture, None, None)?;

    Ok(())
}

fn vec32_to_u8array(vec: &Vec<u32>) -> Vec<u8> {
    let mut result = Vec::new();
    for &value in vec {
        let bytes = value.to_ne_bytes();
        result.extend_from_slice(&bytes);
    }
    result
}

fn render_scene_sdl2(
    camera: &Camera,
    polygons: &[Polygon],
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    width: u32,
    height: u32,
    skip_backfaces: bool,
) -> Result<(), String> {
    // Get the camera state
    let view_matrix = camera.view_matrix();
    let projection_matrix = camera.projection_matrix();

    // println!("Camera position: {:?}, looking: {:?}", camera.position, camera.forward);

    // Clear the canvas with black color
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // Debug: Count processed polygons
    // let mut visible_polygons = 0;
    // let mut total_polygons = 0;

    // Process each polygon
    for polygon in polygons {
        // total_polygons += 1;

        // Skip backfaces
        if skip_backfaces && is_backface(polygon, camera.position) {
            continue;
        }

        let view_vertices: Vec<Point> = polygon
            .vertices
            .iter()
            .map(|vertex| view_matrix.multiply_point(vertex))
            .collect();

        // Check if the entire polygon is behind the camera
        if view_vertices.iter().all(|v| v.z <= 0.1) {
            continue;
        }

        let polygon2d = project_polygon(
            polygon,
            &view_matrix,
            &projection_matrix,
            width as usize,
            height as usize,
        );

        // visible_polygons += 1;

        // Skip empty polygons
        if polygon2d.vertices.len() < 3 {
            println!("Polygon was clipped to < 3 vertices");
            continue;
        } else if polygon2d.vertices.len() == 2 {
            // Set color for line
            let r = ((polygon.color >> 16) & 0xFF) as u8;
            let g = ((polygon.color >> 8) & 0xFF) as u8;
            let b = (polygon.color & 0xFF) as u8;
            let a = ((polygon.color >> 24) & 0xFF) as u8;
            canvas.set_draw_color(Color::RGBA(r, g, b, a));

            canvas.draw_line(
                SDLPoint::new(
                    polygon2d.vertices[0].x as i32,
                    polygon2d.vertices[0].y as i32,
                ),
                SDLPoint::new(
                    polygon2d.vertices[1].x as i32,
                    polygon2d.vertices[1].y as i32,
                ),
            )?;
            continue;
        }

        // Debug: Print projected coordinates
        // println!("Polygon vertices: {:?}", projected.vertices);

        // Extract color from the polygon
        /*
        let r = ((polygon.color >> 16) & 0xFF) as u8;
        let g = ((polygon.color >> 8) & 0xFF) as u8;
        let b = (polygon.color & 0xFF) as u8;
        let a = ((polygon.color >> 24) & 0xFF) as u8;
         */
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));

        // Draw the polygon
        // Create an array of SDL points for drawing
        let sdl_points: Vec<SDLPoint> = polygon2d
            .vertices
            .iter()
            .map(|v| SDLPoint::new(v.x as i32, v.y as i32))
            .collect();

        // Draw polygon outline
        for i in 0..sdl_points.len() {
            let current = sdl_points[i];
            let next = sdl_points[(i + 1) % sdl_points.len()];
            canvas.draw_line(current, next)?;

            // Debug: Draw points with a specific color to ensure visibility
            canvas.set_draw_color(Color::RGB(255, 255, 255));
            canvas.draw_point(current)?;
            canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        }
    }

    // println!("Frame rendered: {}/{} polygons visible", visible_polygons, total_polygons);

    Ok(())
}

/*
fn visualize_bounding_box(polygons: &[Polygon]) -> Vec<Polygon> {
    // Calculate model bounds
    let mut min = Point::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Point::new(f32::MIN, f32::MIN, f32::MIN);

    for polygon in polygons.iter() {
        for vertex in &polygon.vertices {
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);

            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }
    }

    // Calculate center
    let center = Point::new(
        (min.x + max.x) / 2.0,
        (min.y + max.y) / 2.0,
        (min.z + max.z) / 2.0,
    );

    let mut debug_polygons = Vec::new();

    // Create a more visible marker at the center - a square/cube with 6 faces
    let marker_size = 0.3; // Larger size to be more visible

    // Add colored faces (for a cube at the center point)
    // Front face (bright red)
    let mut front = Polygon::new(0xFF0000FF);
    front.add_point(Point::new(
        center.x - marker_size,
        center.y - marker_size,
        center.z - marker_size,
    ));
    front.add_point(Point::new(
        center.x + marker_size,
        center.y - marker_size,
        center.z - marker_size,
    ));
    front.add_point(Point::new(
        center.x + marker_size,
        center.y + marker_size,
        center.z - marker_size,
    ));
    front.add_point(Point::new(
        center.x - marker_size,
        center.y + marker_size,
        center.z - marker_size,
    ));
    debug_polygons.push(front);

    // Add more faces of different colors
    let mut back = Polygon::new(0x00FF00FF); // Green
    back.add_point(Point::new(
        center.x - marker_size,
        center.y - marker_size,
        center.z + marker_size,
    ));
    back.add_point(Point::new(
        center.x - marker_size,
        center.y + marker_size,
        center.z + marker_size,
    ));
    back.add_point(Point::new(
        center.x + marker_size,
        center.y + marker_size,
        center.z + marker_size,
    ));
    back.add_point(Point::new(
        center.x + marker_size,
        center.y - marker_size,
        center.z + marker_size,
    ));
    debug_polygons.push(back);

    println!("Created center marker at {:?}", center);
    println!("Min: {:?}, Max: {:?}", min, max);
    println!("Number of debug polygons: {}", debug_polygons.len());

    debug_polygons
}
 */

fn focus_camera_on_model(camera: &mut Camera, polygons: &[Polygon]) {
    // Calculate model bounds
    let mut min = Point::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Point::new(f32::MIN, f32::MIN, f32::MIN);

    for polygon in polygons.iter() {
        for vertex in &polygon.vertices {
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);

            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }
    }

    // Calculate the center of the model
    let center = Point::new(
        (min.x + max.x) / 2.0,
        (min.y + max.y) / 2.0,
        (min.z + max.z) / 2.0,
    );

    // Calculate vector from camera to model center
    let direction = center - camera.position;

    // Check if camera is already at center
    let length_squared = direction.x * direction.x + direction.y * direction.y + direction.z * direction.z;
    if length_squared < 0.0001 {
        println!("Camera already at model center, not adjusting orientation");
        return;
    }

    // Normalize direction safely
    let length = length_squared.sqrt();
    let normalized = Point::new(
        direction.x / length,
        direction.y / length,
        direction.z / length,
    );

    // Safely calculate pitch (with clamping to avoid domain errors)
    let y_clamped = normalized.y.max(-0.99999).min(0.99999);
    camera.pitch = y_clamped.asin();

    // Calculate yaw safely
    camera.yaw = normalized.z.atan2(normalized.x);

    // Update camera vectors
    camera.update_forward();

    println!("Camera focused on model");
    println!("Model center: {:?}", center);
    println!("Camera position: {:?}", camera.position);
    println!("Direction vector: {:?}", normalized);
    println!("Camera angles: pitch={:.2}, yaw={:.2}", camera.pitch, camera.yaw);
}

fn add_directional_gradient(
    camera: &Camera,
    polygons: &[Polygon],
    framebuffer: &mut Framebuffer,
) {
    // Calculate model center
    let mut min = Point::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Point::new(f32::MIN, f32::MIN, f32::MIN);

    for polygon in polygons.iter() {
        for vertex in &polygon.vertices {
            min.x = min.x.min(vertex.x);
            min.y = min.y.min(vertex.y);
            min.z = min.z.min(vertex.z);

            max.x = max.x.max(vertex.x);
            max.y = max.y.max(vertex.y);
            max.z = max.z.max(vertex.z);
        }
    }

    let center = Point::new(
        (min.x + max.x) / 2.0,
        (min.y + max.y) / 2.0,
        (min.z + max.z) / 2.0,
    );

    // Vector from camera to model center
    let to_center = center - camera.position;

    // Project onto camera's view planes
    let right_comp = to_center.dot(camera.up.cross(camera.forward).normalize());
    let up_comp = to_center.dot(camera.up);
    let forward_comp = to_center.dot(camera.forward);

    // Check if model is likely visible
    if forward_comp > 0.0 {
        let half_fov_h = (camera.fov * camera.aspect_ratio / 2.0).to_radians();
        let half_fov_v = (camera.fov / 2.0).to_radians();

        let h_angle = (right_comp / forward_comp).atan();
        let v_angle = (up_comp / forward_comp).atan();

        if h_angle.abs() < half_fov_h && v_angle.abs() < half_fov_v {
            // Model likely visible, don't show gradient
            return;
        }
    }

    // Get normalized direction
    let length = (right_comp * right_comp + up_comp * up_comp).sqrt();
    let (dir_x, dir_y) = if length > 0.001 {
        (right_comp / length, up_comp / length)
    } else if forward_comp < 0.0 {
        // Object directly behind
        (0.0, -1.0)
    } else {
        return; // Shouldn't happen
    };

    // Apply gradient
    apply_gradient(framebuffer, dir_x, dir_y);
}

fn apply_gradient(framebuffer: &mut Framebuffer, dir_x: f32, dir_y: f32) {
    let width = framebuffer.width;
    let height = framebuffer.height;

    // Calculate center of screen
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;

    // Maximum distance from center to corner
    let max_dist = ((center_x * center_x) + (center_y * center_y)).sqrt();

    // Loop through all pixels
    for y in 0..height {
        for x in 0..width {
            // Calculate position relative to center
            let rel_x = x as f32 - center_x;
            let rel_y = y as f32 - center_y;

            // Calculate dot product with direction
            let dot = rel_x * dir_x + rel_y * dir_y;

            // Normalize to range [0,1] where 1 is in the exact direction
            let intensity = ((dot / max_dist) * 0.5 + 0.5).clamp(0.0, 1.0);

            // Apply directional gradient using a red color with alpha based on intensity
            // Only make edges more intense - center stays more transparent
            let dist_from_center = (rel_x * rel_x + rel_y * rel_y).sqrt() / max_dist;
            let alpha = (intensity * 150.0 * dist_from_center) as u8;

            if alpha > 5 { // Only modify pixels with noticeable gradient
                let index = y * width + x;
                let current_color = framebuffer.pixels[index];

                // Extract current color components
                let r = ((current_color >> 16) & 0xFF) as u8;
                let g = ((current_color >> 8) & 0xFF) as u8;
                let b = (current_color & 0xFF) as u8;

                // Blend with red gradient (0xFF0000)
                let new_r = ((r as u16 * (255 - alpha as u16) + 255 * alpha as u16) / 255) as u8;
                let new_g = ((g as u16 * (255 - alpha as u16)) / 255) as u8;
                let new_b = ((b as u16 * (255 - alpha as u16)) / 255) as u8;

                // Combine into new color (keeping original alpha from framebuffer)
                let new_color = ((current_color & 0xFF000000) |
                                 ((new_r as u32) << 16) |
                                 ((new_g as u32) << 8) |
                                 (new_b as u32));

                framebuffer.pixels[index] = new_color;
            }
        }
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

fn project_polygon(
    polygon: &Polygon,
    view_matrix: &Matrix4x4,
    projection_matrix: &Matrix4x4,
    screen_width: usize,
    screen_height: usize,
) -> Polygon2D {
    let mut vertices_2d: Vec<Point2D> = Vec::new();
    let mut uv_coords_2d: Vec<(f32, f32)> = Vec::new();

    // Transform points to view space
    let mut view_vertices: Vec<Point> = polygon
        .vertices
        .iter()
        .map(|vertex| view_matrix.multiply_point(vertex))
        .collect();

    // Clip polygon to near plane
    let near_plane = 0.1;
    view_vertices = clip_polygon_to_near_plane(&view_vertices, near_plane);

    // Check if the polygon is completely behind the near plane
    if view_vertices.len() < 3 {
        return Polygon2D {
            vertices: vertices_2d,
            uv_coords: uv_coords_2d,
        };
    }

    // Project all remaining vertices
    for (vertex, uv) in view_vertices.iter().zip(&polygon.tex_coords) {
        // Project the vertex using the projection matrix
        let projected = projection_matrix.multiply_point(vertex);

        // Divide for perspective
        let x_ndc = projected.x / projected.z;
        let y_ndc = projected.y / projected.z;

        // Convert to screen coordinates
        let screen_x = ((screen_width as f32 / 2.0) * (1.0 + x_ndc)).round();
        let screen_y = ((screen_height as f32 / 2.0) * (1.0 - y_ndc)).round();

        // Add the point to the 2D vertices
        vertices_2d.push(Point2D {
            x: screen_x,
            y: screen_y,
            z: projected.z, // Depth information does not change
        });

        uv_coords_2d.push(*uv);
    }

    Polygon2D {
        vertices: vertices_2d,
        uv_coords: uv_coords_2d,
    }
}

fn triangulate_ear_clipping(
    polygon: &Polygon2D,
) -> Vec<(
    (Point2D, (f32, f32)),
    (Point2D, (f32, f32)),
    (Point2D, (f32, f32)),
)> {
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
                triangles.push(((prev, prev_uv), (curr, curr_uv), (next, next_uv)));

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
            panic!("Triangulation fehlgeschlagen: Ungültiges oder zu komplexes Polygon!"); // TODO: Panic irgendwie ersetzen mit error oder so
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

    !(has_neg && has_pos)
        || (d1.abs() < f32::EPSILON || d2.abs() < f32::EPSILON || d3.abs() < f32::EPSILON)
}
