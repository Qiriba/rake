mod point;
pub use point::Point;
pub use point::Point2D;

mod matrix4x4;
pub use matrix4x4::Matrix4x4;

mod polygon;
pub use polygon::Polygon;
pub use polygon::Polygon2D;

mod camera;
pub use camera::Camera;

mod framebuffer;
mod object;
mod texture;

pub use framebuffer::Framebuffer;

extern crate winapi;
use crate::texture::Texture;
use lazy_static::lazy_static;
use rayon::prelude::*;
use std::ffi::CString;
use std::io;
use std::io::Write;
use std::path::Path;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winapi::ctypes::c_int;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HBITMAP, HDC, HWND, POINT, RECT};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::wingdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleDC, CreateDIBSection,
    DeleteObject, SRCCOPY, SelectObject,
};
use winapi::um::winuser::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExA, DefWindowProcA, DispatchMessageW,
    GetClientRect, GetCursorPos, GetWindowRect, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage,
    RegisterClassA, SW_SHOW, SetCursorPos, ShowCursor, ShowWindow, TranslateMessage, UpdateWindow,
    WM_DESTROY, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WNDCLASSA, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

static mut WINDOW_WIDTH: usize = 800;
static mut WINDOW_HEIGHT: usize = 600;
static mut POLYGONS: Option<Vec<Polygon>> = None;

lazy_static! {
    static ref CAMERA: Mutex<Camera> = Mutex::new(Camera::new(
        Point::new(0.0, 0.0, -5.0),      // Startposition der Kamera
        Point::new(0.0, 0.0, -1.0),      // Blickrichtung
        Point::new(0.0, 1.0, 0.0),       // "Up"-Vektor
        60.0,                            // Field of View (FOV)
        16f32 / 9f32,                    // Seitenverhältnis
        0.1,                             // Near-Clipping
        100.0                            // Far-Clipping
    ));
}

lazy_static! {
    static ref KEYS: Mutex<[bool; 256]> = Mutex::new([false; 256]);
}

/// Windows-Prozedur - Hier wird das Rendering gesteuert
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_QUIT => {
            PostQuitMessage(0);
            0
        }

        // Wenn das Fenster zerstört wurde
        WM_DESTROY => {
            // Beende die Anwendung
            PostQuitMessage(0);
            0
        }

        WM_KEYDOWN => {
            let key_code = w_param as usize;
            if key_code < 256 {
                let mut keys = KEYS.lock().unwrap();
                if key_code == b'L' as usize {
                    keys[key_code] = !keys[key_code];
                } else {
                    keys[key_code] = true; // Taste als gedrückt markieren
                }
            }
            0
        }

        WM_KEYUP => {
            let key_code = w_param as usize;
            if key_code < 256 {
                let mut keys = KEYS.lock().unwrap();
                if !(key_code == b'L' as usize) {
                    keys[key_code] = false; // Taste als losgelassen markieren
                }
            }
            0
        }

        _ => DefWindowProcA(hwnd, msg, w_param, l_param),
    }
}

unsafe fn handle_input() {
    let keys = KEYS.lock().unwrap();
    let mut camera = CAMERA.lock().unwrap();

    if !keys['L' as usize] {
        process_mouse_input(&mut *camera);
    }
}
unsafe fn process_mouse_input(camera: &mut Camera) {
    let mut cursor_pos = POINT { x: 0, y: 0 };
    GetCursorPos(&mut cursor_pos);

    let window_center_x = WINDOW_WIDTH as i32 / 2;
    let window_center_y = WINDOW_HEIGHT as i32 / 2;

    let delta_x = (cursor_pos.x - window_center_x) as f32;
    let delta_y = (cursor_pos.y - window_center_y) as f32;

    camera.look_around(delta_x, delta_y);
    SetCursorPos(window_center_x, window_center_y);
}

/// Initialisierung eines Fensters
fn init_window() -> HWND {
    unsafe {
        let class_name = CString::new("Rake").unwrap();

        let h_instance = GetModuleHandleA(null_mut());

        let wnd_class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW, // Stil (neu zeichnen bei Fensterbreiten-/Höhenänderung)
            lpfnWndProc: Some(window_proc), // Zeiger auf die Windows-Prozedur
            cbClsExtra: 0,                  // Keine zusätzlichen Bytes in der Fensterklasse
            cbWndExtra: 0,                  // Keine zusätzlichen Bytes im Fenster
            hInstance: h_instance,          // Anwendungsinstanz-Handle
            hIcon: null_mut(),              // Standardsymbol
            hCursor: null_mut(),            // Standard-Cursor
            hbrBackground: (1 + 1) as _,    // Hintergrundfarbe (Weiß)
            lpszMenuName: null_mut(),       // Kein Menü
            lpszClassName: class_name.as_ptr(), // Klassenname
        };

        if RegisterClassA(&wnd_class) == 0 {
            panic!("Fensterklasse konnte nicht registriert werden!");
        }

        let hwnd = CreateWindowExA(
            0,                                      // Keine zusätzlichen Fensterstile
            class_name.as_ptr(),                    // Klassenname
            CString::new("rake").unwrap().as_ptr(), // Fenstertitel
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,       // Standardfensterstil
            CW_USEDEFAULT,                          // Standard-X-Position
            CW_USEDEFAULT,                          // Standard-Y-Position
            WINDOW_WIDTH as c_int,                  // Fensterbreite
            WINDOW_HEIGHT as c_int,                 // Fensterhöhe
            null_mut(),                             // Kein übergeordnetes Fenster
            null_mut(),                             // Kein Menü
            h_instance,                             // Anwendungsinstanz-Handle
            null_mut(),                             // Keine zusätzlichen Anwendungen
        );

        if hwnd.is_null() {
            panic!("Fenster konnte nicht erstellt werden!");
        }

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        hwnd
    }
}

static mut WINDOW_HDC: Option<HDC> = None;

unsafe fn get_window_hdc(hwnd: HWND) -> HDC {
    if let Some(hdc) = WINDOW_HDC {
        return hdc;
    }

    let hdc = winapi::um::winuser::GetDC(hwnd);
    WINDOW_HDC = Some(hdc);
    hdc
}

unsafe fn draw_frame(
    framebuffer: &Framebuffer,
    width: usize,
    height: usize,
    hbitmap: HBITMAP,
    pixels: *mut u32,
    hdc: HDC,
    window_hdc: HDC,
) {
    unsafe {
        std::slice::from_raw_parts_mut(pixels, width * height).copy_from_slice(&framebuffer.pixels);
    }

    let old_object = SelectObject(hdc, hbitmap as *mut _);

    BitBlt(
        window_hdc,
        0,
        0,
        width as i32,
        height as i32,
        hdc,
        0,
        0,
        SRCCOPY,
    );

    // Ressourcenfreigabe
    SelectObject(hdc, old_object);
}

fn update_scene(delta_time: f32) {
    let keys = KEYS.lock().unwrap();
    let mut camera = CAMERA.lock().unwrap();
    if !keys['L' as usize] {
        camera.update_movement(delta_time, &*keys, (0.0, 0.0));
    }
}

fn render_scene(polygons: &Vec<Polygon>, framebuffer: &mut Framebuffer) {
    let camera = CAMERA.lock().unwrap();
    let view_matrix = camera.view_matrix(); // Neuberechnung der View-Matrix nach veränderter camera
    let projection_matrix = camera.projection_matrix();

    framebuffer.clear(); // Framebuffer leeren damit nich sachen übermalt werden

    let projected_polygons: Vec<_> = polygons
        .par_iter()
        .filter_map(|polygon| unsafe {
            if is_backface(polygon, camera.position) {
                return None;
            }

            let projected = polygon::project_polygon(
                polygon,
                &view_matrix,
                &projection_matrix,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            );

            let texture_option = polygon.texture.as_ref().map(|arc| arc.as_ref());

            Some((projected, texture_option, polygon.color))
        })
        .collect();

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

unsafe fn setup_mouse(hwnd: HWND) {
    ShowCursor(0);

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    GetClientRect(hwnd, &mut rect);

    let window_center_x = WINDOW_WIDTH as i32 / 2;
    let window_center_y = WINDOW_HEIGHT as i32 / 2;
    SetCursorPos(window_center_x, window_center_y);
}

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

fn main() {
    unsafe {
        let hwnd = init_window();

        let mut framebuffer = Framebuffer::new(WINDOW_WIDTH, WINDOW_HEIGHT);

        /*
        let mut texture_input = String::new();
        let texture_path = loop {
            print!("Enter texture path: ");
            io::stdout().flush().unwrap();
            texture_input.clear();
            io::stdin().read_line(&mut texture_input).expect("Failed to read input");
            let trimmed = texture_input.trim();
            if Path::new(trimmed).exists() {
                break trimmed;
            } else {
                println!("Invalid path. Please try again.");
            }
        };

        let mut obj_input = String::new();
        let obj_path = loop {
            print!("Enter obj file path: ");
            io::stdout().flush().unwrap();
            obj_input.clear();
            io::stdin().read_line(&mut obj_input).expect("Failed to read input");
            let trimmed = obj_input.trim();
            if Path::new(trimmed).exists() {
                break trimmed;
            } else {
                println!("Invalid path. Please try again.");
            }
        };

        println!("Texture file path: {}", texture_path);
        println!("OBJ file path: {}", obj_path);

        let texture = Texture::from_file(texture_path);
        */

        let texture = Texture::from_file(r#"capsule0.jpg"#);

        let obj_path = r#"capsule.obj"#;

        let (vertices, faces, tex) =
            object::parse_obj_file(obj_path).expect("Failed to load .obj file");

        let mut triangles = object::process_faces(&vertices, &faces, &tex);

        println!("Triangles: {:#?}", triangles.len());

        let shared_texture = Arc::new(texture);
        for triangle in triangles.iter_mut() {
            triangle.set_texture(shared_texture.clone());
        }

        POLYGONS = Some(triangles);

        let mut bitmap_info = create_bitmap_info(&framebuffer);
        let window_hdc = get_window_hdc(hwnd);
        let hdc: HDC = CreateCompatibleDC(window_hdc);
        let mut pixels: *mut u32 = null_mut();
        let mut hbitmap = CreateDIBSection(
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
        let mut rect: RECT = std::mem::zeroed();

        setup_mouse(hwnd);

        loop {
            if GetWindowRect(hwnd, &mut rect) != 0 {
                let new_width = (rect.right - rect.left) as usize;
                let new_height = (rect.bottom - rect.top) as usize;

                if new_width != WINDOW_WIDTH || new_height != WINDOW_HEIGHT {
                    WINDOW_WIDTH = new_width;
                    WINDOW_HEIGHT = new_height;

                    framebuffer.resize(WINDOW_WIDTH, WINDOW_HEIGHT);

                    if hbitmap != null_mut() {
                        DeleteObject(hbitmap as _);
                    }

                    bitmap_info.bmiHeader.biWidth = WINDOW_WIDTH as i32;
                    bitmap_info.bmiHeader.biHeight = -(WINDOW_HEIGHT as i32);

                    let mut new_pixels: *mut u32 = null_mut();
                    hbitmap = CreateDIBSection(
                        hdc,
                        &bitmap_info,
                        0,
                        &mut new_pixels as *mut *mut u32 as *mut *mut _,
                        null_mut(),
                        0,
                    );

                    if hbitmap.is_null() || new_pixels.is_null() {
                        panic!("Failed to recreate DIB section after window resize.");
                    }

                    pixels = new_pixels;
                }
            }
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

            // Zeichne alle Polygone in den framebuffer
            if let Some(ref polygons) = POLYGONS {
                render_scene(polygons, &mut framebuffer);
            }

            // Zeichne den Frame in das Fenster
            draw_frame(
                &framebuffer,
                framebuffer.width,
                framebuffer.height,
                hbitmap,
                pixels,
                hdc,
                window_hdc,
            );
        }
    }
}
