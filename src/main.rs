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

fn update_scene(delta_time: f32) {
    let keys = KEYS.lock().unwrap();
    let mut camera = CAMERA.lock().unwrap();
    camera.update_movement(delta_time, &*keys, (0.0, 0.0));
}

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

/*
fn run_event_loop(
    event_loop: EventLoop<()>,
    window: &Window,
    mut pixels: Pixels,
    mut width: u32,
    mut height: u32,
    polygons: Vec<Polygon>,
) -> Result<(), EventLoopError> {
    let mut last_update: Instant = Instant::now();
    let frame_duration = Duration::from_secs_f32(1.0 / TARGET_FPS);

    // debug tracing
    let mut frame_counter = 0;
    let mut last_fps_print = Instant::now();
    let mut active_events = HashSet::new();

    println!("Starting event loop with window size: {}x{}", width, height);

    // Starting with the mouse NOT captured
    window.set_cursor_visible(true);
    window.set_cursor_grab(winit::window::CursorGrabMode::None).ok();

    event_loop.run(move |event, target| {
        match &event {
            Event::WindowEvent { event, .. } => {
                let event_name = format!("{:?}", event);
                active_events.insert(event_name);
            }
            _ => {}
        }
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(new_size) => {
                    println!("Window resized to: {}x{}", new_size.width, new_size.height);

                    width = new_size.width;
                    height = new_size.height;

                    if width > 8192 || height > 8192 {
                        println!("Window size too large, exiting.");
                        target.exit();
                        return;
                    }

                    if let Err(err) = pixels.resize_buffer(width, height)
                    .and_then(|_| pixels.resize_surface(width, height)){
                        println!("Resize error: {}", err);
                        target.exit();
                    }

                }
                WindowEvent::RedrawRequested => {
                    frame_counter += 1;

                    // render_scene(&polygons, &mut framebuffer);

                    if last_fps_print.elapsed() >= Duration::from_secs(1) {
                        println!("FPS: {}, Active events: {:?}", frame_counter, active_events);
                        frame_counter = 0;
                        last_fps_print = Instant::now();
                        active_events.clear();
                    }

                    // Render logic here
                    let current_time = Instant::now();
                    if current_time.duration_since(last_update) >= frame_duration {
                        let frame = pixels.frame_mut();
                        // let frame_size = frame.len() / 4;
                        // println!("Drawing frame with size: {} pixels", frame_size);
                        frame.chunks_exact_mut(4)
                            .enumerate()
                            .take((width * height) as usize)
                            .for_each(|(i, pixel)| {
                            /*
                            let color = framebuffer.pixels[i];
                            pixels.copy_from_slice(&[
                                ((color >> 16) & 0xFF) as u8,
                                ((color >> 8) & 0xFF) as u8,
                                (color & 0xFF) as u8,
                                ((color >> 24) & 0xFF) as u8,
                                ])};
                            let x = (i % 320) as u8;
                            let y = (i / 320) as u8;
                            pixel.copy_from_slice(&[x, y, 128, 255]); // RGBA
                             */
                            let x = ((i % width as usize) as f32 / width as f32 * 255.0) as u8;
                            let y = ((i / width as usize) as f32 / height as f32 * 255.0) as u8;
                            pixel.copy_from_slice(&[x, y, 0, 255]);
                        });

                        if let Err(err) = pixels.render() {
                            println!("Error rendering pixels: {}", err);
                            target.exit();
                            return;
                        }
                        last_update = current_time;
                    }
                }
                /*
                WindowEvent::CursorMoved { .. } => {
                    // disabled for now
                }
                WindowEvent::Focused(focused) => {
                    if focused && !mouse_captured {
                        mouse_captured = true;
                    }else if !focused && mouse_captured {
                        window.set_cursor_visible(true);
                        mouse_captured = false;
                    }
                }
                 */
                _ => {}//handle_input(&window, &event),
            },
            Event::AboutToWait => {
                // Update logic here

                /*
                let now = Instant::now();
                let delta_time = now.duration_since(last_update);
                update_scene(delta_time.as_secs_f32());
                last_update = now;
                 */

                /*
                let next_frame_time = last_update + frame_duration;
                let now = Instant::now();

                if next_frame_time > now {
                    target.set_control_flow(ControlFlow::WaitUntil(next_frame_time));
                } else {
                    target.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(1)));
                }
                 */

                target.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + frame_duration,
                ));

                window.request_redraw();
            }
            _ => (),
        }
    })
}
 */

/*
fn run_test_event_loop(
    event_loop: EventLoop<()>,
    window: &Window,
    mut pixels: Pixels,
    mut width: u32,
    mut height: u32,
) -> Result<(), EventLoopError> {
    let frame_duration = Duration::from_millis(33); // ~30 FPS
    let mut last_update = Instant::now();

    // No mouse capture, no hidden cursor
    window.set_cursor_visible(true);

    // Don't use any static mutexes during the event loop
    println!("Starting minimal test event loop");

    // Simple solid color for rendering
    let mut color: u8 = 0;

    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(new_size) => {
                    println!("Window resized to: {}x{}", new_size.width, new_size.height);
                    width = new_size.width;
                    height = new_size.height;
                    // Simple resize without error checking for testing
                    let _ = pixels.resize_buffer(width, height);
                    let _ = pixels.resize_surface(width, height);
                }
                WindowEvent::RedrawRequested => {
                    // Only render at fixed intervals
                    let now = Instant::now();
                    if now.duration_since(last_update) >= frame_duration {
                        println!("Rendering frame at: {:?}", now.elapsed());

                        // Fill with solid color that changes
                        color = color.wrapping_add(1);
                        let frame = pixels.frame_mut();
                        for pixel in frame.chunks_exact_mut(4) {
                            pixel.copy_from_slice(&[color, color, color, 255]);
                        }

                        // Simple render without error checking
                        let _ = pixels.render();
                        last_update = now;
                    }
                }
                // Ignore all other input events completely
                _ => {},
            },
            Event::AboutToWait => {
                // Simple wait without complicated logic
                target.set_control_flow(ControlFlow::WaitUntil(
                    Instant::now() + Duration::from_millis(16)
                ));
                window.request_redraw();
            }
            _ => (),
        }
    })
}
 */

/*
fn main1() {
    unsafe {
        match init_window() {
            Ok((event_loop, window)) => {
                let (pixels, width, height) = init_rendering(&window);
                println!(
                    "Window initialized successfully with size: {}x{}",
                    width, height
                );
                if let Err(err) = run_test_event_loop(event_loop, &window, pixels, width, height) {
                    println!("Error in event loop: {}", err);
                }
                /*
                if let Err(e) = pixels.render() {
                    println!("Initial render failed: {}", e);
                    return;
                }
                 */

                /*let polygons = Some(vec![]);

                if let Err(err) = run_event_loop(
                    event_loop,
                    &window,
                    pixels,
                    width,
                    height,
                    polygons.unwrap(),
                ) {
                    println!("Error in event loop: {}", err);
                }
                 */
            }
            Err(err) => {
                println!("Error initializing window: {}", err);
            }
        }

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
        /*
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
         */

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
 */

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point as SDLPoint;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("SDL2 Test", WIDTH, HEIGHT)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    let frame_duration = Duration::from_secs_f32(1.0 / TARGET_FPS);
    let mut last_update = Instant::now();
    let mut color: u8 = 0;

    println!("Starting SDL2 render loop");

    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
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
        last_update = Instant::now();

        // Update color
        color = color.wrapping_add(1);

        // Render gradient pattern
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let r = (x % 255) as u8;
                let g = (y % 255) as u8;
                let b = color;

                canvas.set_draw_color(Color::RGB(r, g, b));
                canvas.draw_point(SDLPoint::new(x as i32, y as i32))?;
            }
        }

        canvas.present();
    }

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
