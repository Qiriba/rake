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

pub use framebuffer::Framebuffer;

extern crate winapi;
use std::cmp::{PartialEq};
use std::ptr::{null_mut};
use std::ffi::CString;
use std::{ptr};
use std::sync::Mutex;
use std::time::Instant;
use winapi::shared::windef::{HBITMAP, HDC, HWND, };
use winapi::shared::minwindef::{LRESULT, LPARAM, UINT, WPARAM};
use winapi::um::wingdi::{
    CreateCompatibleDC, CreateDIBSection, SelectObject, BitBlt,
    SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
};
use winapi::um::winuser::{CreateWindowExA, DefWindowProcA, DispatchMessageA, PeekMessageA, RegisterClassA, TranslateMessage, UpdateWindow, ShowWindow, WNDCLASSA, MSG, WM_PAINT, WM_QUIT, WS_OVERLAPPEDWINDOW, WS_VISIBLE, SW_SHOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, PM_REMOVE, GetMessageW, DispatchMessageW, PeekMessageW, WM_KEYDOWN, WM_KEYUP, PostQuitMessage, WM_DESTROY};
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

unsafe fn handle_input() {
    let keys = KEYS.lock().unwrap();

    let mut camera = CAMERA.lock().unwrap(); // Erlaubt Schreibzugriff

    if let Some(ref mut polygons) = POLYGONS {

        // Bewegung

        if keys['W' as usize] {
            camera.move_forward(0.1); // W-Taste bewegt die Kamera vorwärts
        }
        if keys['S' as usize] {
            camera.move_backward(0.1); // S-Taste bewegt die Kamera rückwärts
        }
        if keys['D' as usize] {
            camera.strafe_right(0.1); // D-Taste bewegt die Kamera nach rechts
        }
        if keys['A' as usize] {
            camera.strafe_left(0.1); // A-Taste bewegt die Kamera nach links
        }


        // Rotation
        if keys[0x51] { // 'Q' - Drehe um +10° (X-Achse)
            camera.look_left(0.01);
        }
        if keys[0x52] { // 'R' - Drehe um +10° (Y-Achse)
            for polygon in polygons.iter_mut() {
                let rotation = (0.0, 10.0_f32.to_radians(), 0.0);
                polygon.rotate_around_center(rotation);
            }
        }
        if keys[0x45] { // 'E' - Drehe um +10° (Z-Achse)
            camera.look_right(0.01);
        }

        // Skalierung
        if keys[0x5A] { // 'Z' - Vergrößern (x1.1)
            for polygon in polygons.iter_mut() {
                let scale = (1.1, 1.1, 1.0);
                polygon.transform_full((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), scale);
            }
        }
        if keys[0x58] { // 'X' - Verkleinern (x0.9)
            for polygon in polygons.iter_mut() {
                let scale = (0.9, 0.9, 1.0);
                polygon.transform_full((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), scale);
            }
        }
    }
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
fn update_scene() {
    // Hier kann Logik zur Szenenaktualisierung hinzugefügt werden
}

fn render_scene(polygons: &Vec<Polygon>, framebuffer: &mut Framebuffer) {
    let camera = CAMERA.lock().unwrap(); // Kamera-Instanz
    let view_matrix = camera.view_matrix(); // Neuberechnung der View-Matrix
    let projection_matrix = camera.projection_matrix(); // Projektion

    framebuffer.clear(); // Framebuffer leeren

    for polygon in polygons {
        //println!("Polygon: {:?}", polygon);

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

fn main() {
    unsafe{
        //let mut framebuffer : Vec<u32>= vec![0xFF000000; WINDOW_WIDTH * WINDOW_HEIGHT]; // Black background
        let mut framebuffer = Framebuffer::new(WINDOW_WIDTH,WINDOW_HEIGHT);

        let texture = Texture::from_file("C:\\Users\\Tobias\\Pictures\\texture.jpg");
        const FOCAL_LENGTH: f32 = 800.0;
        POLYGONS = Some(vec![{
            let mut polygon = Polygon::new(0xFFFFFFFF); // Weißes Polygon
            polygon.add_point(Point::new(-1.0, -1.0, 5.0));
            polygon.add_point(Point::new(1.0, -1.0, 5.0));
            polygon.add_point(Point::new(0.0, 1.0, 5.0));
            polygon.add_texture(texture.clone());
            polygon.set_tex_coords(vec![
    (0.0, 1.0), // unten-links
    (1.0, 1.0), // unten-rechts
    (1.0, 0.0), // oben-rechts
    (0.0, 0.0), // oben-links
]
            );
            polygon
        }]);


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


       let mut msg: MSG = std::mem::zeroed();

        loop {
            let event_start = Instant::now();

            // Nachrichten abarbeiten (ohne blockieren)
            //User Input etc
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) > 0 {
                if msg.message == WM_QUIT {
                    return; // Beendet die Nachrichtenschleife
                }
                TranslateMessage(&msg); // Übersetze Tastatureingaben
                DispatchMessageW(&msg); // Nachricht verarbeiten
            }
            handle_input();

            /*
            let current_time = Instant::now();
            let frame_time = current_time - previous_time; // Dauer des Frames
            previous_time = current_time;

            // (2) FPS berechnen
            let frame_time_seconds = frame_time.as_secs_f32(); // Konvertiere Frame-Dauer in Sekunden
            let fps = 1.0 / frame_time_seconds;

            // FPS ausgeben
            println!("Frames Per Second: {:.2}", fps);
            */

            //let event_start = Instant::now();

            // Clear den Framebuffer, um die alten Frames zu überschreiben ansonsten bleiben die alten im Bild
            framebuffer.clear();

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
            let event_time = event_start.elapsed();
            println!("Zeit für alles: {:.2?}", event_time);

        }

    }
   //cleanup();
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

    for (vertex, uv) in polygon.vertices.iter().zip(&polygon.tex_coords) {
        // 1. Transformiere den Vertex in den View-Space
        let view_transformed = view_matrix.multiply_point(vertex);

        // 2. Punkt muss vor der Kamera liegen
        if view_transformed.z > 0.0 {
            // 3. Projiziere den Punkt in den Clip-Space
            let projected = projection_matrix.multiply_point(&view_transformed);

            // 4. Perspektivische Division (Normalisierung)
            let x_ndc = projected.x / projected.z;
            let y_ndc = projected.y / projected.z;

            // 5. Konvertiere in Bildschirmkoordinaten
            let screen_x = ((screen_width as f32 / 2.0) * (1.0 + x_ndc)).round();
            let screen_y = ((screen_height as f32 / 2.0) * (1.0 - y_ndc)).round();

            // Füge den Punkt in 2D-Liste ein
            vertices_2d.push(Point2D {
                x: screen_x,
                y: screen_y,
                z: projected.z, // Behalte Tiefeninformation
            });

            // UV-Texturkoordinate unverändert weitergeben
            uv_coords_2d.push(*uv);
        }
    }

    // Rückgabe des projizierten Polygons in 2D (mit UV-Koordinaten, falls vorhanden)
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
