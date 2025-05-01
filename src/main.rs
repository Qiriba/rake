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

use image::error::ImageFormatHint::Name;
use std::cmp::PartialEq;
use std::ffi::CString;
use std::fmt::Debug;
use std::io::Write;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::{env, io};
use std::collections::HashSet;
/*
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey, PhysicalKey},
    window::WindowBuilder,
};
 */

use crate::texture::Texture;
use lazy_static::lazy_static;
use pixels::wgpu::naga::SwizzleComponent::W;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};

use crate::matrix4x4::Matrix4x4;
use rayon::prelude::*;
use winit::error::EventLoopError;
// use winit::window::Window;

const WINDOW_WIDTH: f64 = 800.0;
const WINDOW_HEIGHT: f64 = 600.0;
const TARGET_FPS: f32 = 30.0;
static mut POLYGONS: Option<Vec<Polygon>> = None;

lazy_static! {
    static ref CAMERA: Mutex<Camera> = Mutex::new(Camera::new(
        Point::new(0.0, 0.0, 5.0),      // Starting position
        Point::new(0.0, 0.0, -1.0),       // View direction
        Point::new(0.0, 1.0, 0.0),       // "Up"-vector
        60.0,                            // Field of View (FOV)
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32, // Seitenverhältnis
        0.1,                             // Near clipping
        100.0                            // Far clipping
    ));
}

lazy_static! {
    static ref KEYS: Mutex<[bool; 256]> = Mutex::new([false; 256]);
}

/*
/// Windows-Prozedur - Hier wird das Rendering gesteuert
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    return match msg {
        WM_QUIT => {
            PostQuitMessage(0);
            0
        }

        // Wenn das Fenster zerstört wurde
        WM_DESTROY => {
            // Beende die Anwendung
            PostQuitMessage(0);
            return 0;
        }

        WM_KEYDOWN => {
            let key_code = w_param as usize;
            if key_code < 256 {
                let mut keys = KEYS.lock().unwrap();
                keys[key_code] = true; // Taste als gedrückt markieren
            }
            0
        }

        WM_KEYUP => {
            let key_code = w_param as usize;
            if key_code < 256 {
                let mut keys = KEYS.lock().unwrap();
                keys[key_code] = false; // Taste als losgelassen markieren
            }
            0
        }

        _ => DefWindowProcA(hwnd, msg, w_param, l_param),
    };

}
 */

/*
fn handle_input(window: &Window, event: &WindowEvent) {
    match event {
        WindowEvent::KeyboardInput {
            device_id: _,
            event,
            is_synthetic: _,
        } => {
            if let Key::Character(c) = &event.logical_key {
                let mut keys = KEYS.lock().unwrap();
                let is_pressed = event.state == ElementState::Pressed;
                match c.as_str() {
                    "w" | "a" | "s" | "d" => {
                        KEYS.lock().unwrap()[c.as_bytes()[0] as usize] = is_pressed;
                    }
                    "q" if is_pressed => CAMERA.lock().unwrap().look_right(),
                    "e" if is_pressed => CAMERA.lock().unwrap().look_left(),
                    _ => (),
                }
            }
        }
        WindowEvent::CursorMoved { position, .. } => {
            let mut camera = CAMERA.lock().unwrap();
            let window_center_x = WINDOW_WIDTH as i32 / 2;
            let window_center_y = WINDOW_HEIGHT as i32 / 2;

            let delta_x = (position.x as i32 - window_center_x) as f32;
            let delta_y = (position.y as i32 - window_center_y) as f32;

            camera.look_around(delta_x, delta_y);
            // Cursor auf center zurücksetzen
            window
                .set_cursor_position(winit::dpi::PhysicalPosition::new(
                    window_center_x,
                    window_center_y,
                ))
                .unwrap();
        }
        _ => (),
    }
}
 */

/// Initialisierung eines Fensters
/*
fn init_window() -> Result<(EventLoop<()>, Window), EventLoopError> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Rake")
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_visible(true)
        .with_position(winit::dpi::PhysicalPosition::new(100.0, 100.0))
        .with_resizable(true)
        .build(&event_loop)?;
    Ok((event_loop, window))
}
*/

/*
fn init_rendering(window: &Window) -> (Pixels, u32, u32) {
    let size = window.inner_size();
    let width = size.width;
    let height = size.height;
    let surface_texture = SurfaceTexture::new(width, height, window);
    let pixels = Pixels::new(width, height, surface_texture).unwrap();
    (pixels, width, height)
}
 */

/*
static mut WINDOW_HDC: Option<HDC> = None;

unsafe fn get_window_hdc(hwnd: HWND) -> HDC {
    if let Some(hdc) = WINDOW_HDC {
        return hdc;
    }

    let hdc = winapi::um::winuser::GetDC(hwnd);
    WINDOW_HDC = Some(hdc);
    hdc
}
 */

/*
unsafe fn draw_frame(framebuffer: &Framebuffer, hbitmap: HBITMAP, pixels: *mut u32, hdc: HDC, window_hdc: HDC) {

    unsafe {
        std::slice::from_raw_parts_mut(pixels, width * height)
            .copy_from_slice(&framebuffer.pixels);
    }

    let old_object = SelectObject(hdc, hbitmap as *mut _);


    BitBlt(
        window_hdc,
        0,
        0,
        framebuffer.width as i32,
        framebuffer.height as i32,
        hdc,
        0,
        0,
        SRCCOPY,
    );



    // Ressourcenfreigabe
    SelectObject(hdc, old_object);
}
 */

/*
fn render_scene(polygons: &Vec<Polygon>, framebuffer: &mut Framebuffer) {
    let camera = CAMERA.lock().unwrap();
    let view_matrix = camera.view_matrix(); // Neuberechnung der View-Matrix nach veränderter camera
    let projection_matrix = camera.projection_matrix();

    framebuffer.clear(); // Framebuffer leeren damit nich sachen übermalt werden

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
                WINDOW_WIDTH as usize,
                WINDOW_HEIGHT as usize,
            );

            // Gib Option<&Texture> weiter, falls vorhanden
            let texture_option = polygon.texture.as_ref();

            Some((projected, texture_option, polygon.color))
        })
        .collect();

    // Jetzt seriell ins framebuffer rendern
    for (projected, texture, color) in projected_polygons {
        framebuffer.draw_polygon(&projected, texture, color);
    }
}
 */

fn is_backface(polygon: &Polygon, camera_position: Point) -> bool {
    if polygon.vertices.len() < 3 {
        return true; // Kann kein gültiges Polygon sein wenn weniger als 3 Ecken
    }

    let edge1 = polygon.vertices[1] - polygon.vertices[0];
    let edge2 = polygon.vertices[2] - polygon.vertices[0];
    let normal = edge1.cross(edge2).normalize();

    let view_direction = (camera_position - polygon.vertices[0]).normalize();
    normal.dot(view_direction) < 0.0
}

/*
fn setup_mouse(window: &Window) {
    // Hide the cursor
    window.set_cursor_visible(false);

    // Center the cursor in the window
    let window_center_x = WINDOW_WIDTH as i32 / 2;
    let window_center_y = WINDOW_HEIGHT as i32 / 2;
    window
        .set_cursor_position(winit::dpi::PhysicalPosition::new(
            window_center_x,
            window_center_y,
        ))
        .unwrap();
}
 */

/*
unsafe fn create_bitmap_info(framebuffer: &Framebuffer) -> BITMAPINFO {
    let mut bitmap_info: BITMAPINFO = std::mem::zeroed();
    bitmap_info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bitmap_info.bmiHeader.biWidth = framebuffer.width as i32;
    bitmap_info.bmiHeader.biHeight = -(framebuffer.height as i32); // Negative Höhe damit Top-Down-Rendering erfolgt
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biBitCount = 32; // (ARGB)
    bitmap_info.bmiHeader.biCompression = BI_RGB;
    bitmap_info
}
 */

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point as SDLPoint;

const INITIAL_WIDTH: u32 = 320;
const INITIAL_HEIGHT: u32 = 240;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

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

    let polygons = load_obj("./example.obj")?;

    let mut mouse_captured = false;
    sdl_context.mouse().set_relative_mouse_mode(false);

    let mut skip_backfaces = true;

    println!("Starting SDL2 render loop");

    'running: loop {
        // Mouse movement tracking
        let mut mouse_delta = (0.0f32, 0.0f32);

        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::Window { win_event, .. } => {
                    if let sdl2::event::WindowEvent::Resized(w, h) = win_event {
                        width = w as u32;
                        height = h as u32;
                        println!("Window resized to: {}x{}", width, height);
                    }
                },
                Event::MouseMotion { xrel, yrel, .. } => {
                    if mouse_captured {
                        mouse_delta.0 = xrel as f32 * 2.5;
                        mouse_delta.1 = yrel as f32 * 2.5;
                    }
                },
                Event::KeyDown { keycode: Some(key), .. } => {
                    // Update key state
                    let mut keys = KEYS.lock().unwrap();
                    match key {
                        Keycode::W => keys['W' as usize] = true,
                        Keycode::A => keys['A' as usize] = true,
                        Keycode::S => keys['S' as usize] = true,
                        Keycode::D => keys['D' as usize] = true,
                        Keycode::Space => keys['V' as usize] = true,
                        Keycode::B => {
                            skip_backfaces = !skip_backfaces;
                            println!("Skip backfaces: {}", skip_backfaces);
                        },
                        Keycode::Tab => {
                            mouse_captured = !mouse_captured;
                            sdl_context.mouse().set_relative_mouse_mode(mouse_captured);
                        },
                        _ => {}
                    }
                },
                Event::KeyUp { keycode: Some(key), .. } => {
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
                },
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
            let mut camera = CAMERA.lock().unwrap();
            let keys = KEYS.lock().unwrap();
            camera.update_movement(delta_time, &*keys, mouse_delta);
        }

        // Render the scene
        render_scene_sdl2(&polygons, &mut canvas, width, height, skip_backfaces)?;

        canvas.present();

    }

    Ok(())
}

fn load_obj(path: &str) -> Result<Vec<Polygon>, String> {
    // Read the file to string
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read OBJ file: {}", e))?;

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
                let x = parts.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                let y = parts.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                let z = parts.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                vertices.push(Point::new(x, y, z));
            },
            "vt" => {
                // Parse texture coordinates (vt u v)
                let u = parts.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                let v = parts.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                tex_coords.push((u, v));
            },
            "vn" => {
                // Parse normals (vn x y z)
                let nx = parts.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                let ny = parts.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                let nz = parts.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
                normals.push(Point::new(nx, ny, nz));
            },
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
                        polygon.tex_coords = face_tex_coords;
                    }

                    polygons.push(polygon);
                }
            },
            _ => {} // Ignore other lines
        }
    }

    // Return the loaded polygons
    if polygons.is_empty() {
        Err("No valid polygons found in the OBJ file".to_string())
    } else {
        println!("Loaded {} vertices and {} polygons from {}", vertices.len(), polygons.len(), path);
        Ok(polygons)
    }
}

fn load_test_cube() -> Vec<Polygon> {
    // Create a simple cube
    let mut polygons = Vec::new();

    // Create a cube centered at (0, 0, -3) instead of (0, 0, 0)
    // This places it 2 units in front of the camera (which is at z=5)

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

    // Add four more faces to complete the cube
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

fn render_scene_sdl2(
    polygons: &[Polygon],
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    width: u32,
    height: u32,
    skip_backfaces: bool
) -> Result<(), String> {
    // Get the camera state
    let camera = CAMERA.lock().unwrap();
    let view_matrix = camera.view_matrix();
    let projection_matrix = camera.projection_matrix();

    // println!("Camera position: {:?}, looking: {:?}", camera.position, camera.forward);

    // Clear the canvas with black color
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // Debug: Count processed polygons
    let mut visible_polygons = 0;
    let mut total_polygons = 0;

    // Process each polygon
    for polygon in polygons {
        total_polygons += 1;

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
        if view_vertices.iter().all(|v| v.z <= 0.1){
            continue;
        }

        let mut vertices_2d: Vec<Point2D> = Vec::new();

        for vertex in &view_vertices {
            // Skip points behind the camera (z <= 0)
            if vertex.z < 0.1 {
                continue; // Skip vertices behind the near plane
            }

            // Project the point into clip space
            let projected = projection_matrix.multiply_point(vertex);

            // Perspective division
            let x_ndc = projected.x / projected.z;
            let y_ndc = projected.y / projected.z;

            // Convert to screen coordinates
            let screen_x = ((width as f32 / 2.0) * (1.0 + x_ndc)).round();
            let screen_y = ((height as f32 / 2.0) * (1.0 - y_ndc)).round();

            vertices_2d.push(Point2D { x: screen_x, y: screen_y, z: 0.0 });
        }

        visible_polygons += 1;

        // Skip empty polygons
        if vertices_2d.len() < 3 {
            println!("Polygon was clipped to < 3 vertices");
            continue;
        }

        // Debug: Print projected coordinates
        // println!("Polygon vertices: {:?}", projected.vertices);

        // Extract color from the polygon
        let r = ((polygon.color >> 16) & 0xFF) as u8;
        let g = ((polygon.color >> 8) & 0xFF) as u8;
        let b = (polygon.color & 0xFF) as u8;
        let a = ((polygon.color >> 24) & 0xFF) as u8;
        canvas.set_draw_color(Color::RGBA(r, g, b, a));

        // Draw the polygon
        // Create an array of SDL points for drawing
        let sdl_points: Vec<SDLPoint> = vertices_2d.iter()
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
            canvas.set_draw_color(Color::RGBA(r, g, b, a));
        }

        // Optional: Add filled polygon rendering here once the outlines are working
    }

    // println!("Frame rendered: {}/{} polygons visible", visible_polygons, total_polygons);

    Ok(())

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

/// A polygon defined as a list of vertices.
fn project_polygon(
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

    // Prüfe, ob das Polygon noch existiert (kann nach Clipping ungültig werden)
    if view_vertices.len() < 3 {
        return Polygon2D {
            vertices: vertices_2d,
            uv_coords: uv_coords_2d,
        };
    }

    // Projiziere alle übriggebliebenen Punkte
    for (vertex, uv) in view_vertices.iter().zip(&polygon.tex_coords) {
        // 1. Projiziere den Punkt in den Clip-Space
        let projected = projection_matrix.multiply_point(vertex);

        // 2. Perspektivische Division
        let x_ndc = projected.x / projected.z;
        let y_ndc = projected.y / projected.z;

        // 3. Konvertiere in Bildschirmkoordinaten
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

fn process_faces(vertices: &Vec<Point>, faces: &Vec<Vec<usize>>) -> Vec<Polygon> {
    faces
        .par_iter()
        .filter_map(|face| {
            if face.len() < 3 {
                return None; // Ungültiges Face überspringen
            }

            // Punkte extrahieren
            let points: Vec<Point> = face.iter().map(|&i| vertices[i]).collect();

            // Neues Polygon erstellen
            let mut polygon = Polygon::new(0xFFFFFFFF);
            for point in &points {
                polygon.add_point(*point);
            }

            Some(polygon)
        })
        .collect()
}
