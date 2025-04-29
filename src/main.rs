mod point;
pub use point::Point;
pub use point::Point2D;

mod polygon;
pub use polygon::Polygon;
pub use polygon::Polygon2D;

mod camera;
pub use camera::Camera;

mod framebuffer;
mod texture;
mod object;
mod matrix4x4;

pub use framebuffer::Framebuffer;

use std::cmp::{PartialEq};
use std::{env, io};
use std::ptr::{null_mut};
use std::ffi::CString;
use std::fmt::Debug;
use std::io::Write;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use image::error::ImageFormatHint::Name;
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey, PhysicalKey},
    window::WindowBuilder,
};

use lazy_static::lazy_static;
use pixels::{Pixels, SurfaceTexture};
use crate::texture::Texture;

use rayon::prelude::*;
use winit::error::EventLoopError;
use winit::window::Window;
use crate::matrix4x4::Matrix4x4;

const WINDOW_WIDTH: f64 = 800.0;
const WINDOW_HEIGHT: f64 = 600.0;
static mut POLYGONS: Option<Vec<Polygon>> = None;

lazy_static! {
    static ref CAMERA: Mutex<Camera> = Mutex::new(Camera::new(
        Point::new(0.0, 0.0, -5.0),      // Startposition der Kamera
        Point::new(0.0, 0.0, -1.0),       // Blickrichtung
        Point::new(0.0, 1.0, 0.0),       // "Up"-Vektor
        60.0,                            // Field of View (FOV)
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32, // Seitenverhältnis
        0.1,                             // Near-Clipping
        100.0                            // Far-Clipping
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

fn handle_input(window: &Window, event: &WindowEvent) {

    match event {
         WindowEvent::KeyboardInput{
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
        WindowEvent::CursorMoved { position, ..} => {
            let mut camera = CAMERA.lock().unwrap();
            let window_center_x = WINDOW_WIDTH as i32 / 2;
            let window_center_y = WINDOW_HEIGHT as i32 / 2;

            let delta_x = (position.x as i32 - window_center_x) as f32;
            let delta_y = (position.y as i32 - window_center_y) as f32;

            camera.look_around(delta_x, delta_y);
            // Cursor auf center zurücksetzen
            window.set_cursor_position(winit::dpi::PhysicalPosition::new(window_center_x, window_center_y)).unwrap();
        }
        _ => (),
    }
}

/// Initialisierung eines Fensters
fn init_window() -> Result<(EventLoop<()>, Window), EventLoopError> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Rake")
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_visible(true)
        .with_position(winit::dpi::PhysicalPosition::new(100.0, 100.0))
        .build(&event_loop)?;
    Ok((event_loop, window))
}

fn init_rendering(window: &Window) -> Pixels {
    let size = window.inner_size();
    let surface_texture = SurfaceTexture::new(size.width, size.height, window);
    let pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();
    pixels
}

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


 fn update_scene(delta_time: f32) {

    let keys = KEYS.lock().unwrap();
    let mut camera = CAMERA.lock().unwrap();
    camera.update_movement(delta_time, &*keys, (0.0, 0.0));

}

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
    let edge2 = polygon.vertices[2]  - polygon.vertices[0] ;
    let normal = edge1.cross(edge2).normalize();



    let view_direction = (camera_position - polygon.vertices[0]).normalize();
    normal.dot(view_direction) < 0.0
}

fn setup_mouse(window: &Window) {
    // Hide the cursor
    window.set_cursor_visible(false);

    // Center the cursor in the window
    let window_center_x = WINDOW_WIDTH as i32 / 2;
    let window_center_y = WINDOW_HEIGHT as i32 / 2;
    window.set_cursor_position(winit::dpi::PhysicalPosition::new(window_center_x, window_center_y)).unwrap();
}

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

fn run_event_loop(event_loop: EventLoop<()>, window: &Window, mut pixels: Pixels, polygons: Vec<Polygon>) -> Result<(), EventLoopError> {
    let mut last_update: Instant = Instant::now();

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::RedrawRequested => {

                    // Render logic here
                    let frame = pixels.frame_mut();
                    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                        let x = (i % 320) as u8;
                        let y = (i / 320) as u8;
                        pixel.copy_from_slice(&[x, y, 128, 255]); // RGBA
                    }
                    pixels.render().unwrap();
                },
                _ => handle_input(&window, &event),
            },
            Event::AboutToWait => {
                // Update logic here

                let now = Instant::now();
                let delta_time = now.duration_since(last_update);
                update_scene(delta_time.as_secs_f32());
                last_update = now;

                window.request_redraw();
            }
            _ => (),
        }
    })
}

fn main() {
    unsafe{
        let (event_loop, window) = init_window().unwrap();
        let pixels = init_rendering(&window);

        pixels.render().unwrap();

        /*
        // Prompt for the first file path
        print!("Enter texture path: ");
        io::stdout().flush().unwrap(); // Ensure prompt is shown
        let mut first_path = String::new();
        io::stdin()
            .read_line(&mut first_path)
            .expect("Failed to read input");
        let first_path = first_path.trim(); // Remove newline

        // Prompt for the second file path
        print!("Enter obj file path: ");
        io::stdout().flush().unwrap();
        let mut second_path = String::new();
        io::stdin()
            .read_line(&mut second_path)
            .expect("Failed to read input");
        let second_path = second_path.trim();

        // Print to confirm
        println!("First file path: {}", first_path);
        println!("Second file path: {}", second_path);
         */
        let texture_path = r#"/home/emil/Downloads/shrek-meme.jpg"#;

        let obj_path = r#"./example.obj"#;

        let texture = Texture::from_file(texture_path);
        // Lade die .obj-Daten
        let (vertices, faces) = object::parse_obj_file(obj_path).expect("Failed to load .obj file");

        let mut triangles = process_faces(&vertices, &faces);
        println!("Triangles: {:#?}", triangles.len());
        for triangle in triangles.iter_mut() {
            triangle.set_texture(texture.clone());
            triangle.set_tex_coords(vec![
                (0.0, 1.0), // unten-links
                (1.0, 1.0), // unten-rechts
                (0.5, 0.0), // oben-rechts
            ]
            );
        }

        let polygons = Some(
            triangles
        );

        run_event_loop(event_loop, &window, pixels, polygons.unwrap()).expect("aaaa");

        /*
        let bitmap_info = create_bitmap_info(&framebuffer);

        let window_hdc = unsafe { get_window_hdc(hwnd) };
        let hdc: HDC = CreateCompatibleDC(window_hdc);

        let mut pixels: *mut u32 = null_mut();
        let hbitmap = CreateDIBSection(
            hdc,
            &bitmap_info,
            0,
            &mut pixels as *mut *mut u32 as *mut *mut _,
            null_mut(),
            0,
        );
        const UPDATE_RATE: u64 = 60;
        const TIMESTEP: f32 = 1.0 / UPDATE_RATE as f32;
        let mut previous_time = Instant::now();
        let mut lag = 0.0;

        let mut msg: MSG = std::mem::zeroed();

        setup_mouse(hwnd);

        loop {
            let current_time = Instant::now();
            let delta_time = (current_time - previous_time).as_secs_f32();
            previous_time = current_time;

            lag += delta_time;


            //Nachrichten abarbeiten ohne zu blockieren
            //User Input etc
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) > 0 {
                if msg.message == WM_QUIT {
                    return;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            handle_input();

            while lag >= TIMESTEP {
                update_scene(TIMESTEP);
                lag -= TIMESTEP;
            }

            //let event_start = Instant::now();
            //let event_time = event_start.elapsed();
            //println!("Zeit für framebuffclear: {:.2?}", event_time);


            // Zeichne alle Polygone in den framebuffer
            unsafe {
                if let Some(ref polygons) = POLYGONS {
                    render_scene(polygons, &mut framebuffer);
                }
            };

            // Zeichne den Frame in das fenster
            draw_frame(&framebuffer, WINDOW_WIDTH, WINDOW_HEIGHT, hbitmap, pixels, hdc, window_hdc);
        }
         */
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
            panic!("Triangulation fehlgeschlagen: Ungültiges oder zu komplexes Polygon!");                      // TODO: Panic irgendwie ersetzen mit error oder so
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
