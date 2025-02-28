extern crate winapi;

use std::cmp::{max, min, Ordering, PartialEq};
use std::ptr::{null, null_mut};
use std::ffi::CString;
use std::ptr;
use std::time::Instant;
use winapi::shared::windef::{HBITMAP, HDC, HWND, RECT};
use winapi::shared::minwindef::{LRESULT, LPARAM, UINT, WPARAM};
use winapi::um::wingdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BitBlt,
    SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
};
use winapi::um::winuser::{CreateWindowExA, DefWindowProcA, DispatchMessageA, GetClientRect, GetMessageA, PeekMessageA, RegisterClassA, TranslateMessage, UpdateWindow, ShowWindow, WNDCLASSA, MSG, WM_PAINT, WM_QUIT, WS_OVERLAPPEDWINDOW, WS_VISIBLE, SW_SHOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, PM_REMOVE, GetMessageW, DispatchMessageW, PeekMessageW};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::ctypes::c_int;

const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;

/// Windows-Prozedur - Hier wird das Rendering gesteuert
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_QUIT => {
            return 0
        }
        _ => return DefWindowProcA(hwnd, msg, w_param, l_param),
    }
    0
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
            CString::new("My Rust Window").unwrap().as_ptr(), // Fenstertitel
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

/// Framebuffer in das Fenster zeichnen
unsafe fn draw_frame(hwnd: HWND, framebuffer: &Framebuffer, width: usize, height: usize) {
    let hdc: HDC = CreateCompatibleDC(null_mut());

    // Framebuffer Setup (Bitmap)
    let mut bitmap_info: BITMAPINFO = std::mem::zeroed();
    bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bitmap_info.bmiHeader.biWidth = width as i32;
    bitmap_info.bmiHeader.biHeight = -(height as i32); // Negative Höhe, damit Top-Down-Rendering erfolgt
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biBitCount = 32; // (ARGB)
    bitmap_info.bmiHeader.biCompression = BI_RGB;

    let mut pixels: *mut u32 = null_mut();
    let hbitmap: HBITMAP = CreateDIBSection(
        hdc,
        &bitmap_info,
        0,
        &mut pixels as *mut *mut u32 as *mut *mut _,
        null_mut(),
        0,
    );

    if !pixels.is_null() {
        // Kopiere den Framebuffer in die Bitmap-Pixel-Daten
        std::ptr::copy_nonoverlapping(framebuffer.pixels.as_ptr(), pixels, width * height);
    }

    let old_object = SelectObject(hdc, hbitmap as *mut _);

    // Zeichne die Bitmap auf das Fenster
    let window_hdc = winapi::um::winuser::GetDC(hwnd);
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
    DeleteObject(hbitmap as *mut _);
    DeleteDC(hdc);
}

/// Framebuffer manipulieren (Szene rendern)
fn render_scene(framebuffer: &mut [u32], width: usize, height: usize) {
    for y in 0..height {
        for x in 0..width {
            let red = (x * 255 / width) as u32;    // Farbverlauf horizontal (Rot)
            let green = (y * 255 / height) as u32; // Farbverlauf vertikal (Grün)
            let blue = 128u32;                    // Konstant Blau

            framebuffer[y * width + x] = 0xFF000000 | (red << 16) | (green << 8) | blue;
        }
    }
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
        // Initialisiere das Fenster
        let hwnd = init_window();

        //let mut framebuffer : Vec<u32>= vec![0xFF000000; WINDOW_WIDTH * WINDOW_HEIGHT]; // Black background
        let mut framebuffer = Framebuffer::new(WINDOW_WIDTH,WINDOW_HEIGHT);

        let mut zbuffer = vec![f32::INFINITY; WINDOW_WIDTH * WINDOW_HEIGHT]; // Z-buffer initialization

        // Ziel-FPS festlegen
        let target_fps: u32 = 60;
        let frame_duration = 1_000_000_000 / target_fps as u64; // Dauer eines Frames in Nanosekunden

        let mut running = true;
        //while running {
            let frame_start = current_time_ns();

            // Verarbeite Nachrichten
            //running = handle_window_events();

            // Aktualisiere die Szene (falls nötig)
            //update_scene();

            //draw_frame(hwnd,&mut framebuffer,WINDOW_WIDTH,WINDOW_HEIGHT);

            // Synchronisiere die Framerate
            //wait_for_next_frame(frame_start, frame_duration);
        //}
        let width = 100;
        let height = 100;


        framebuffer.clear();

        let focal_length = 800.0;
        let mut polygons = vec![
            Polygon { vertices: vec![
                Point::new(-1.0, -1.0, 8.0), // Links unten (näher)
                Point::new(1.0, -1.0, 8.0), // Rechts unten (leicht entfernt)
                Point::new(1.0, 1.0, 8.0), // Rechts oben (weiter entfernt)
                Point::new(-1.0, 1.0, 8.0), // Links oben (leicht entfernt)


            ],
                color: 0xFFFFFF00
            }
            /*
            ,
            Polygon { vertices: vec![
                Point::new(-2.0, -2.0, 6.0),
                Point::new(0.0, -2.0, 6.0),
                Point::new(-1.0, 0.0, 6.0),

            ],
                color: 0xFFFFFFFF
            },*/
        ];

        let rotation_speed = 0.0174533; // 1 Grad in Radiant (1° = π/180)

        let mut previous_time = Instant::now(); // Zeitpunkt vor dem Start des Frames

        let mut counter: u64 = 0;
        loop {
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
            // Drehe das Polygon
            counter += 1;
            polygons[0].rotate_z(rotation_speed);

            // Clear den Framebuffer, um die alten Frames zu überschreiben
            framebuffer.clear();

            // Zeichne alle Polygone
            for polygon in &polygons {
                let polygon_2d = project_polygon(&polygon, focal_length, framebuffer.width, framebuffer.height);
                framebuffer.draw_polygon(&polygon_2d, polygon.color);
            }

            // Zeichne den Frame
            draw_frame(hwnd, &framebuffer, WINDOW_WIDTH, WINDOW_HEIGHT);

            // Nachrichten abarbeiten (ohne blockieren)
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) > 0 {
                TranslateMessage(&msg); // Übersetze Tastatureingaben
                DispatchMessageW(&msg); // Nachricht verarbeiten
            }

            // Beende die Schleife, wenn das Fenster geschlossen wird
            if !handle_window_events() {
                break;
            }
        }
        let span = Instant::now() - previous_time;
        print!("{}", counter / span.as_secs());


    }
   //cleanup();
}

/// A 3D point.
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point { x, y , z}
    }
}
fn project_point_3d_to_2d(point: Point, focal_length: f32) -> (f32, f32) {
    // Perspective projection formula
    let x_proj = point.x * focal_length / point.z;
    let y_proj = point.y * focal_length / point.z;


    (x_proj, y_proj)
}

/// A polygon defined as a list of vertices.
#[derive(Debug)]
pub struct Polygon {
    pub vertices: Vec<Point>,
    pub color: u32
}

impl Polygon {
    /// Create a new empty polygon.
    pub fn new(colorout: u32) -> Self {
        Polygon {
            vertices: Vec::new(),
            color: colorout
        }
    }

    /// Add a point to the polygon.
    pub fn add_point(&mut self, point: Point) {
        self.vertices.push(point);
    }

    /// Get the number of points in the polygon.
    pub fn num_points(&self) -> usize {
        self.vertices.len()
    }

    pub fn rotate_z(&mut self, angle_radians: f32) {
        // Wende die Rotation auf jeden Punkt an
        for vertex in self.vertices.iter_mut() {
            *vertex = rotate_point_around_z(*vertex, angle_radians);
        }
    }


}

fn project_polygon(
    polygon: &Polygon,
    focal_length: f32,
    screen_width: usize,
    screen_height: usize,
) -> Polygon2D {
    let mut vertices_2d: Vec<Point2D> = Vec::new();

    for vertex in &polygon.vertices {
        if vertex.z > 0.0 {
            // Project point to 2D space
            let (x_proj, y_proj) = project_point_3d_to_2d(*vertex, focal_length);

            // Convert to screen space
            let screen_x = ((screen_width as f32 / 2.0) + x_proj).round();
            let screen_y = ((screen_height as f32 / 2.0) - y_proj).round();


            vertices_2d.push(Point2D {
                x: screen_x,
                y: screen_y,
                z: vertex.z
            });
        }
    }
    Polygon2D { vertices: vertices_2d }
}

#[derive(Copy, Clone, Debug)]
struct Point2D {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Debug)]
struct Polygon2D {
    vertices: Vec<Point2D>,
}

impl PartialEq for Point2D {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x &&
        self.y == other.y &&
        (self.z - other.z).abs() < f32::EPSILON
    }
}

fn rotate_point_around_z(point: Point, angle_radians: f32) -> Point {
    let cos_theta = angle_radians.cos();
    let sin_theta = angle_radians.sin();

    Point {
        x: point.x * cos_theta - point.y * sin_theta,
        y: point.x * sin_theta + point.y * cos_theta,
        z: point.z, // z bleibt unverändert
    }
}

struct Framebuffer {
    width: usize,
    height: usize,
    pixels: Vec<u32>,  // Store the color values (e.g., 0xRRGGBBAA)
    z_buffer: Vec<f32>, // Store depth values for each pixel
}

impl Framebuffer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height], // Schwarzes Bild
            z_buffer: vec![f32::INFINITY; width * height], // Z-Buffer initial auf "unendlich"
        }
    }

    fn clear(&mut self) {
        self.pixels.fill(0); // Framebuffer zurücksetzen
        self.z_buffer.fill(f32::INFINITY); // Z-Buffer zurücksetzen
    }

        fn draw_polygon(&mut self, polygon: &Polygon2D, color: u32) {
            if polygon.vertices.len() < 3 {
                return; // Kein gültiges Polygon
            }

            let triangles = triangulate_ear_clipping(polygon);

            // Render jedes Dreieck separat
            for (v0, v1, v2) in triangles {
                self.rasterize_triangle(v0, v1, v2, color);
            }


    }

    fn rasterize_triangle(&mut self, v0: Point2D, v1: Point2D, v2: Point2D, color: u32) {
        // Bounding box des Dreiecks

        let v0 = snap_to_pixel(v0);
        let v1 = snap_to_pixel(v1);
        let v2 = snap_to_pixel(v2);

        let min_x = v0.x.min(v1.x).min(v2.x).max(0.0) as i32;
        let max_x = v0.x.max(v1.x).max(v2.x).min((self.width - 1) as f32) as i32;
        let min_y = v0.y.min(v1.y).min(v2.y).max(0.0)as i32;
        let max_y = v0.y.max(v1.y).max(v2.y).min((self.height - 1) as f32)as i32;

        // Iterator über alle Pixel innerhalb der Bounding-Box
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                // Berechne Baryzentrische Koordinaten
                if let Some((w0, w1, w2)) = barycentric_coordinates(x as f32, y as f32, v0, v1, v2) {
                    // Interpolierter Tiefenwert (z) für den Pixel
                    let z = w0 * v0.z + w1 * v1.z + w2 * v2.z;

                    // Z-Buffer-Test
                    let index = (y as usize * self.width + x as usize);
                    if z < self.z_buffer[index] {
                        self.z_buffer[index] = z; // Tiefenwert aktualisieren
                        self.pixels[index] = color; // Farbwert setzen
                    }
                }
            }
        }
    }
}

// Hilfsfunktion: Baryzentrische Koordinaten
fn barycentric_coordinates(
    px: f32, py: f32,
    v0: Point2D, v1: Point2D, v2: Point2D,
) -> Option<(f32, f32, f32)> {
    let epsilon = 1e-5; // Toleranzwert

    let det = (v1.y - v2.y) * (v0.x - v2.x) + (v2.x - v1.x) * (v0.y - v2.y);
    if det.abs() < epsilon {
        return None; // Degeneriertes Dreieck
    }

    let l1 = ((v1.y - v2.y) * (px - v2.x) + (v2.x - v1.x) * (py - v2.y)) / det;
    let l2 = ((v2.y - v0.y) * (px - v2.x) + (v0.x - v2.x) * (py - v2.y)) / det;
    let l3 = 1.0 - l1 - l2;

    // Toleranzregelung an den Grenzen
    if l1 >= -epsilon && l2 >= -epsilon && l3 >= -epsilon {
        Some((l1.max(0.0), l2.max(0.0), l3.max(0.0)))
    } else {
        None
    }
}


fn triangulate_ear_clipping(polygon: &Polygon2D) -> Vec<(Point2D, Point2D, Point2D)> {
    let mut triangles = Vec::new();
    let mut vertices = polygon.vertices.clone(); // Kopiere die Punkte des Polygons

    if polygon.vertices.len() == 4 {
        // Rechteck/Quadrat besonderer Fall – einfache Zwei-Dreiecks-Zerlegung
        return vec![
            (polygon.vertices[0], polygon.vertices[1], polygon.vertices[2]),
            (polygon.vertices[2], polygon.vertices[3], polygon.vertices[0]),
        ];
    }

    ensure_ccw(&mut vertices);


    while vertices.len() > 3 {
        let mut ear_found = false;

        // Finde ein Ohr
        for i in 0..vertices.len() {
            let prev = vertices[(i + vertices.len() - 1) % vertices.len()]; // Vorheriger Punkt
            let curr = vertices[i]; // Aktueller Punkt
            let next = vertices[(i + 1) % vertices.len()]; // Nächster Punkt

            if is_ear(prev, curr, next, &vertices) {
                // Ein Ohr wurde gefunden
                triangles.push((prev, curr, next));
                vertices.remove(i);
                ear_found = true;
                break;
            }
            else {
            }

        }

        if !ear_found {
            panic!("Triangulation fehlgeschlagen – ungültiges oder komplexes Polygon?");
        }
    }

    // Füge das letzte verbleibende Dreieck hinzu
    if vertices.len() == 3 {
        triangles.push((vertices[0], vertices[1], vertices[2]));
    }

    triangles
}


fn snap_to_pixel(point: Point2D) -> Point2D {
    Point2D {
        x: point.x.round(),
        y: point.y.round(),
        z: point.z, // z kann unangetastet bleiben
    }
}

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

fn is_ccw(p1: Point2D, p2: Point2D, p3: Point2D) -> bool {

    let cross_product = (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x);


    if cross_product > 0.0 {
        true // Gegen den Uhrzeigersinn
    } else {
        false
    }

}

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

fn ensure_ccw(vertices: &mut Vec<Point2D>) {
    if !is_polygon_ccw(vertices) {
        vertices.reverse();
    }
}

fn is_point_in_triangle(p: Point2D, a: Point2D, b: Point2D, c: Point2D) -> bool {
    let det = |p1: Point2D, p2: Point2D, p3: Point2D| -> f32 {
        (p2.x - p1.x) as f32 * (p3.y - p1.y) as f32 - (p2.y - p1.y) as f32 * (p3.x - p1.x) as f32
    };

    let d1 = det(p, a, b);
    let d2 = det(p, b, c);
    let d3 = det(p, c, a);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos) || (d1.abs() < f32::EPSILON || d2.abs() < f32::EPSILON || d3.abs() < f32::EPSILON)

}
