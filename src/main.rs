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
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, io};
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
pub const TARGET_FPS: f32 = 60.0;
static mut POLYGONS: Option<Vec<Polygon>> = None;

lazy_static! {
    static ref CAMERA: Mutex<Camera> = Mutex::new(Camera::new(
        Point::new(0.0, 0.0, 5.0),      // Starting position
        Point::new(0.0, 0.0, 1.0),       // View direction
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
use sdl2::pixels::{Color, PixelFormatEnum};
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

    let mut framebuffer = Framebuffer::new(width as usize, height as usize);

    let mut polygons = load_obj("./capsule.obj").unwrap_or_else(|e| {
        println!("Error loading OBJ file: {}", e);
        println!("Falling back to default cube");
        load_test_cube()
    });

    let texture = Texture::from_file("./capsule0.jpg");

    let shared_texture = Arc::new(texture);

    // Assign texture to polygons
    // if let Some(tex) = texture {
        for polygon in &mut polygons {
            polygon.texture = Some(shared_texture.clone()) // Some(tex.clone());
        }
    // }

    normalize_model(&mut polygons, 2.0);

    let mut bbox_polygons: Vec<Polygon> = Vec::new();

    let mut mouse_captured = false;
    sdl_context.mouse().set_relative_mouse_mode(false);

    let mut skip_backfaces = true;
    let mut show_bbox = false;

    println!("Starting SDL2 render loop");

    'running: loop {
        // Mouse movement tracking
        let mut mouse_delta = (0.0f32, 0.0f32);

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
                        Keycode::Comma => focus_camera_on_model(&polygons),
                        Keycode::Space => keys['V' as usize] = true,
                        Keycode::B => {
                            skip_backfaces = !skip_backfaces;
                            println!("Skip backfaces: {}", skip_backfaces);
                        }
                        Keycode::P => {
                            show_bbox = !show_bbox;
                            if show_bbox {
                                bbox_polygons = visualize_bounding_box(&polygons);
                                println!("Showing bounding box");
                            } else {
                                println!("Hiding bounding box");
                            }
                        }
                        Keycode::Tab => {
                            mouse_captured = !mouse_captured;
                            sdl_context.mouse().set_relative_mouse_mode(mouse_captured);
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
            let mut camera = CAMERA.lock().unwrap();
            let keys = KEYS.lock().unwrap();
            camera.update_movement(delta_time, &*keys, mouse_delta);
        }

        if true {
            render_scene_sdl2(&bbox_polygons, &mut canvas, width, height, skip_backfaces)?;
        } else {
            render_scene(&polygons, &mut framebuffer);
            fb_to_canvas(&framebuffer, &mut canvas).expect("Error converting framebuffer to canvas");
        }

        // Render the scene
        /*if show_bbox {
        } else {
            render_scene_sdl2(&polygons, &mut canvas, width, height, skip_backfaces)?;
        }*/

        canvas.present();
    }

    Ok(())
}

fn load_obj(path: &str) -> Result<Vec<Polygon>, String> {
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
                vertices.push(Point::new(x, -z, y));
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
    polygons: &[Polygon],
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    width: u32,
    height: u32,
    skip_backfaces: bool,
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
        if view_vertices.iter().all(|v| v.z <= 0.1) {
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
        } else if vertices_2d.len() == 2 {
            // Set color for line
            let r = ((polygon.color >> 16) & 0xFF) as u8;
            let g = ((polygon.color >> 8) & 0xFF) as u8;
            let b = (polygon.color & 0xFF) as u8;
            let a = ((polygon.color >> 24) & 0xFF) as u8;
            canvas.set_draw_color(Color::RGBA(r, g, b, a));

            canvas.draw_line(
                SDLPoint::new(vertices_2d[0].x as i32, vertices_2d[0].y as i32),
                SDLPoint::new(vertices_2d[1].x as i32, vertices_2d[1].y as i32),
            )?;
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

fn focus_camera_on_model(polygons: &[Polygon]) {
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

    // Calculate the exact center of the model
    let center = Point::new(
        (min.x + max.x) / 2.0,
        (min.y + max.y) / 2.0,
        (min.z + max.z) / 2.0,
    );

    let mut camera = CAMERA.lock().unwrap();

    // Calculate direction from camera position to model center
    let direction = center - camera.position;
    let direction_length = (direction.x.powi(2) + direction.y.powi(2) + direction.z.powi(2)).sqrt();

    // Normalize direction
    let direction = Point::new(
        direction.x / direction_length,
        direction.y / direction_length,
        direction.z / direction_length,
    );

    // Calculate pitch and yaw to look directly at model center
    camera.pitch = (-direction.y).asin();
    camera.yaw = (-direction.x).atan2(-direction.z);

    // Update camera vectors
    camera.update_forward();

    println!("Camera rotated to view model");
    println!("Model center: {:?}", center);
    println!("Camera position: {:?}", camera.position);
    println!("Looking direction: {:?}", direction);
    println!(
        "Camera angles: pitch={:.2}, yaw={:.2}",
        camera.pitch, camera.yaw
    );
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
