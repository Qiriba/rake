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
mod texture;
mod object;

pub use framebuffer::Framebuffer;

extern crate winapi;
use std::cmp::{PartialEq};
use std::ptr::{null_mut};
use std::ffi::CString;
use std::{ptr};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use winapi::shared::windef::{HBITMAP, HDC, HWND, POINT, RECT};
use winapi::shared::minwindef::{LRESULT, LPARAM, UINT, WPARAM};
use winapi::um::wingdi::{
    CreateCompatibleDC, CreateDIBSection, SelectObject, BitBlt,
    SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
};
use winapi::um::winuser::{CreateWindowExA, DefWindowProcA, DispatchMessageA, PeekMessageA, RegisterClassA, TranslateMessage, UpdateWindow, ShowWindow, WNDCLASSA, MSG, WM_PAINT, WM_QUIT, WS_OVERLAPPEDWINDOW, WS_VISIBLE, SW_SHOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, PM_REMOVE, GetMessageW, DispatchMessageW, PeekMessageW, WM_KEYDOWN, WM_KEYUP, PostQuitMessage, WM_DESTROY, GetAsyncKeyState, GetCursorPos, ScreenToClient, SetCursorPos, ShowCursor, GetClientRect, ClipCursor};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::ctypes::c_int;
use lazy_static::lazy_static;
use crate::texture::Texture;

const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;
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
    0
}

unsafe fn handle_input(hwnd: HWND) {
    let mut keys = KEYS.lock().unwrap();
    let mut camera = CAMERA.lock().unwrap();

    // Handle mouse movement (look around)
    process_mouse_input(hwnd, &mut *camera);

    // Check key state for movement
    if GetAsyncKeyState(b'A' as i32) < 0 {
        keys[b'A' as usize] = true;
    } else {
        keys[b'A' as usize] = false;
    }

    if GetAsyncKeyState(b'D' as i32) < 0 {
        keys[b'D' as usize] = true;
    } else {
        keys[b'D' as usize] = false;
    }

    if GetAsyncKeyState(b'W' as i32) < 0 {
        keys[b'W' as usize] = true;
    } else {
        keys[b'W' as usize] = false;
    }

    if GetAsyncKeyState(b'S' as i32) < 0 {
        keys[b'S' as usize] = true;
    } else {
        keys[b'S' as usize] = false;
    }

    // Jump input (Space)
    if GetAsyncKeyState(b'V' as i32) < 0 {
        keys[b'V' as usize] = true;
    } else {
        keys[b'V' as usize] = false;
    }

    if keys[b'Q' as usize] {
        let mut camera = CAMERA.lock().unwrap();
        camera.look_left();
    }
    if keys[b'E' as usize] {
        let mut camera = CAMERA.lock().unwrap();
        camera.look_right();
    }

}
unsafe fn process_mouse_input(hwnd: HWND, camera: &mut Camera) {
    let mut cursor_pos = POINT { x: 0, y: 0 };

    // Get the current cursor position in screen coordinates
    GetCursorPos(&mut cursor_pos);
    //ScreenToClient(hwnd, &mut cursor_pos);


    // Get the size of the window (center point)
    let window_center_x = crate::WINDOW_WIDTH as i32 / 2;
    let window_center_y = crate::WINDOW_HEIGHT as i32 / 2;

    // Calculate delta movement
    let delta_x = (cursor_pos.x - window_center_x) as f32;
    let delta_y = (cursor_pos.y - window_center_y) as f32;

    camera.look_around(delta_x, delta_y);
    // Recenter the cursor to the middle of the window
    SetCursorPos(window_center_x, window_center_y);
}

/// Initialisierung eines Fensters
fn init_window() -> HWND {
    unsafe {
        // Fensterklassenname definieren
        let class_name = CString::new("MyWindowClass").unwrap();

        // Modul-Handle abrufen
        let h_instance = GetModuleHandleA(null_mut());

        // Fensterklasse definieren und registrieren
        let wnd_class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,     // Stil (neu zeichnen bei Fensterbreiten-/Höhenänderung)
            lpfnWndProc: Some(window_proc),    // Zeiger auf die Windows-Prozedur
            cbClsExtra: 0,                     // Keine zusätzlichen Bytes in der Fensterklasse
            cbWndExtra: 0,                     // Keine zusätzlichen Bytes im Fenster
            hInstance: h_instance,             // Anwendungsinstanz-Handle
            hIcon: null_mut(),                 // Standardsymbol
            hCursor: null_mut(),               // Standard-Cursor
            hbrBackground: (1 + 1) as _,       // Hintergrundfarbe (Weiß)
            lpszMenuName: null_mut(),          // Kein Menü
            lpszClassName: class_name.as_ptr(), // Klassenname
        };

        if RegisterClassA(&wnd_class) == 0 {
            panic!("Fensterklasse konnte nicht registriert werden!");
        }

        // Fenster erstellen
        let hwnd = CreateWindowExA(
            0,                                   // Keine zusätzlichen Fensterstile
            class_name.as_ptr(),                 // Klassenname
            CString::new("rake").unwrap().as_ptr(), // Fenstertitel
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,    // Standardfensterstil
            CW_USEDEFAULT,                       // Standard-X-Position
            CW_USEDEFAULT,                       // Standard-Y-Position
            WINDOW_WIDTH as c_int,               // Fensterbreite
            WINDOW_HEIGHT as c_int,              // Fensterhöhe
            null_mut(),                          // Kein übergeordnetes Fenster
            null_mut(),                          // Kein Menü
            h_instance,                          // Anwendungsinstanz-Handle
            null_mut(),                          // Keine zusätzlichen Anwendungen
        );

        if hwnd.is_null() {
            panic!("Fenster konnte nicht erstellt werden!");
        }

        // Fenster anzeigen und aktualisieren
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
    WINDOW_HDC = Some(hdc); // Store the HDC for future use
    hdc
}

/// Framebuffer in das Fenster zeichnen
unsafe fn draw_frame(framebuffer: &Framebuffer, width: usize, height: usize, hbitmap: HBITMAP, pixels: *mut u32, hdc: HDC, window_hdc: HDC) {

    //let event_start = Instant::now();

    unsafe {
        std::slice::from_raw_parts_mut(pixels, width * height)
            .copy_from_slice(&framebuffer.pixels);
    }

    //let event_time = event_start.elapsed();
    //println!("Zeit für copy from slice: {:.2?}", event_time);

    let old_object = SelectObject(hdc, hbitmap as *mut _);

    //let event_start = Instant::now();

    // Zeichne die Bitmap auf das Fenster
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

   // let event_time = event_start.elapsed();
    //println!("Zeit für BitBlt und window_hdc: {:.2?}", event_time);

    //let event_start = Instant::now();


    // Ressourcenfreigabe
    SelectObject(hdc, old_object);

    //let event_time = event_start.elapsed();
    //println!("Zeit für Resourcenfreigabe: {:.2?}", event_time);
}


/// Nachrichtenschleife und Handling
fn handle_window_events() -> bool {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();

        while PeekMessageA(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
            if msg.message == WM_QUIT {
                return false;
            }
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
    true
}



/// Dummy-Funktion: Szene aktualisieren
 fn update_scene(delta_time: f32) {

    let keys = KEYS.lock().unwrap();
    let mut camera = CAMERA.lock().unwrap();
    camera.update_movement(delta_time, &*keys, (0.0, 0.0));

}

fn render_scene(polygons: &Vec<Polygon>, framebuffer: &mut Framebuffer) {
    let camera = CAMERA.lock().unwrap(); // Kamera-Instanz
    let view_matrix = camera.view_matrix(); // Neuberechnung der View-Matrix
    let projection_matrix = camera.projection_matrix(); // Projektion
    let camera_position = camera.position;

    framebuffer.clear(); // Framebuffer leeren

    for polygon in polygons {
        //println!("Polygon: {:?}", polygon);
        if is_backface(polygon, camera_position) {
            continue; // Überspringe unsichtbare Rückseiten
        }

        // Projiziere jedes Polygon
        let projected_polygon = project_polygon(
            polygon,
            &view_matrix,
            &projection_matrix,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );
        framebuffer.draw_polygon(&projected_polygon, Option::from(&polygon.texture), polygon.color); // Zeichne Polygon
    }

}
fn is_backface(polygon: &Polygon, camera_position: Point) -> bool {
    if polygon.vertices.len() < 3 {
        return true; // Kann kein gültiges Polygon sein
    }

    // Berechne die Normale
    let normal = calculate_normal(
        polygon.vertices[0],
        polygon.vertices[1],
        polygon.vertices[2],
    );

    fn calculate_normal(p0: Point, p1: Point, p2: Point) -> Point {
        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        edge1.cross(edge2).normalize()
    }

    // Definiere eine Blickrichtung
    let view_direction = (camera_position - polygon.vertices[0]).normalize();

    // Prüfe die Ausrichtung: Rückseiten werden ausgeschlossen
    normal.dot(view_direction) < 0.0
}


/// Dummy-Funktion: Aktuelle Zeit in Nanosekunden zurückgeben
fn current_time_ns() -> u64 {
    0
}

/// Dummy-Funktion: Warte für Framerate-Synchronisation
fn wait_for_next_frame(_frame_start: u64, _frame_duration: u64) {}

/// Dummy-Funktion: Ressourcen bereinigen
fn cleanup() {
    println!("Ressourcen bereinigt.");
}
unsafe fn setup_mouse(hwnd: HWND) {
    // Hide the cursor
    ShowCursor(0);

    // Confine the cursor to the window
    let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    GetClientRect(hwnd, &mut rect);
    ClipCursor(&rect as *const RECT);

    // Center the cursor in the window
    let window_center_x = crate::WINDOW_WIDTH as i32 / 2;
    let window_center_y = crate::WINDOW_HEIGHT as i32 / 2;
    SetCursorPos(window_center_x, window_center_y);
}

fn main() {
    unsafe{
        //let mut framebuffer : Vec<u32>= vec![0xFF000000; WINDOW_WIDTH * WINDOW_HEIGHT]; // Black background
        let mut framebuffer = Framebuffer::new(WINDOW_WIDTH,WINDOW_HEIGHT);

        let texture = Texture::from_file(r#"C:\Users\Tobias\Pictures\4x_1.png"#);

        let obj_path = r#"C:\Users\Tobias\RustroverProjects\rake\example.obj"#; // Pfad zur OBJ-Datei

        // Lade die .obj-Daten
        let (vertices, faces) = object::parse_obj_file(obj_path).expect("Failed to load .obj file");

        let mut triangles = process_faces(&vertices, &faces);

        for triangle in triangles.iter_mut() {
            triangle.add_texture(texture.clone());
            triangle.set_tex_coords(vec![
                (0.0, 1.0), // unten-links
                (1.0, 1.0), // unten-rechts
                (0.5, 0.0), // oben-rechts
            ]
            );
        }
        const FOCAL_LENGTH: f32 = 800.0;
        POLYGONS = Some(/*vec![{
            let mut polygon = Polygon::new(0xFFFFFFFF); // Weißes Polygon
            polygon.add_point(Point::new(-1.0, -1.0, 5.0));
            polygon.add_point(Point::new(1.0, -1.0, 5.0));
            polygon.add_point(Point::new(0.0, 1.0, 5.0));
            polygon.add_texture(texture.clone());
            polygon.set_tex_coords(vec![
    (0.0, 1.0), // unten-links
    (1.0, 1.0), // unten-rechts
    (0.5, 0.0), // oben-rechts
]
            );
            polygon
        }]*/

        triangles
        );


        let hwnd = init_window();

        // Framebuffer Setup (Bitmap)
        let mut bitmap_info: BITMAPINFO = std::mem::zeroed();
        bitmap_info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bitmap_info.bmiHeader.biWidth = framebuffer.width as i32;
        bitmap_info.bmiHeader.biHeight = -(framebuffer.height as i32); // Negative Höhe, damit Top-Down-Rendering erfolgt
        bitmap_info.bmiHeader.biPlanes = 1;
        bitmap_info.bmiHeader.biBitCount = 32; // (ARGB)
        bitmap_info.bmiHeader.biCompression = BI_RGB;
        let window_hdc = unsafe { get_window_hdc(hwnd) };
        let hdc: HDC = CreateCompatibleDC(window_hdc);

        let mut pixels: *mut u32 = null_mut();
        let hbitmap: HBITMAP = CreateDIBSection(
            hdc,
            &bitmap_info,
            0,
            &mut pixels as *mut *mut u32 as *mut *mut _,
            null_mut(),
            0,
        );

        const UPDATE_RATE: u64 = 60; // Fixed logic updates per second
        const TIMESTEP: f32 = 1.0 / UPDATE_RATE as f32;
        let mut previous_time = Instant::now();
        let mut lag = 0.0;

       let mut msg: MSG = std::mem::zeroed();
        // Initialize timing
        let mut last_frame_time = Instant::now(); // Time at the start of the frame

        setup_mouse(hwnd);

        loop {
            let current_time = Instant::now();
            let delta_time = (current_time - previous_time).as_secs_f32();
            previous_time = current_time;

            lag += delta_time;


            // Nachrichten abarbeiten (ohne blockieren)
            //User Input etc
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) > 0 {
                if msg.message == WM_QUIT {
                    return; // Beendet die Nachrichtenschleife
                }
                TranslateMessage(&msg); // Übersetze Tastatureingaben
                DispatchMessageW(&msg); // Nachricht verarbeiten
            }

            handle_input(hwnd);

            while lag >= TIMESTEP {
                update_scene(TIMESTEP); // Fixed timestep logic updates
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

            // Zeichne den Frame
            draw_frame(&framebuffer, WINDOW_WIDTH, WINDOW_HEIGHT, hbitmap, pixels, hdc, window_hdc);

        }

    }
   //cleanup();
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

    // Clippe gegen die Near-Plane (je nach Bedarf auch Far-Plane etc.)
    let near_plane = 0.1; // Nahe Grenze
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

        // Füge den Punkt in die 2D-Liste ein
        vertices_2d.push(Point2D {
            x: screen_x,
            y: screen_y,
            z: projected.z, // Tiefeninformation beibehalten
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

    // Sicherstellen, dass das Polygon in CCW-Reihenfolge (Counter-Clockwise) vorliegt
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

fn process_faces(vertices: &Vec<Point>, faces: &Vec<Vec<usize>>) -> Vec<Polygon> {
    let mut polygons = Vec::new();

    for face in faces {
        if face.len() < 3 {
            // Überspringe ungültige Faces
            continue;
        }

        // Extrahiere die zugehörigen Punkte (Vertices) des Faces
        let points: Vec<Point> = face.iter().map(|&index| vertices[index]).collect();
        let mut polygon = Polygon::new(0xFFFFFFFF);
        for point in &points {
            polygon.add_point(*point);
        }

        // Füge die Punkte als ein neues Polygon ein
        polygons.push(polygon);
    }

    polygons
}


