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
use winapi::um::winuser::{CreateWindowExA, DefWindowProcA, DispatchMessageA, PeekMessageA, RegisterClassA, TranslateMessage, UpdateWindow, ShowWindow, WNDCLASSA, MSG, WM_PAINT, WM_QUIT, WS_OVERLAPPEDWINDOW, WS_VISIBLE, SW_SHOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, PM_REMOVE, GetMessageW, DispatchMessageW, PeekMessageW, WM_KEYDOWN, WM_KEYUP};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::ctypes::c_int;
use lazy_static::lazy_static;

const WINDOW_WIDTH: usize = 800;
const WINDOW_HEIGHT: usize = 600;
static mut POLYGONS: Option<Vec<Polygon>> = None;

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
            0
        }
        WM_KEYDOWN => {
            let key_code = w_param as usize;
            if key_code < 256 {
                let mut keys = KEYS.lock().unwrap();
                keys[key_code] = true; // Taste als gedrückt markieren
            }

            if let Some(ref mut polygons) = POLYGONS {
                match w_param as i32 {
                    0x41 => { // 'A' - Nach links
                        for polygon in polygons {
                            let translation = (-0.1, 0.0, 0.0);
                            polygon.transform_full(translation, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
                        }
                    }
                    0x44 => { // 'D' - Nach rechts
                        for polygon in polygons {
                            let translation = (0.1, 0.0, 0.0);
                            polygon.transform_full(translation, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
                        }
                    }
                    0x57 => { // 'W' - Nach oben
                        for polygon in polygons {
                            let translation = (0.0, 0.1, 0.0);
                            polygon.transform_full(translation, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
                        }
                    }
                    0x53 => { // 'S' - Nach unten
                        for polygon in polygons {
                            let translation = (0.0, -0.1, 0.0);
                            polygon.transform_full(translation, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
                        }
                    }

                    0x51 => { // 'Q' - Drehe um +10° (X-Achse)
                        for polygon in polygons {
                            let rotation = (10.0_f32.to_radians(), 0.0, 0.0);

                            polygon.rotate_around_center(rotation);
                        }
                    }
                    0x52 => { // 'R' - Drehe um +10° (Y-Achse)
                        for polygon in polygons {
                            let rotation = (0.0, 10.0_f32.to_radians(), 0.0);
                            polygon.rotate_around_center(rotation);
                        }
                    }
                    0x45 => { // 'E' - Drehe um +10° (Z-Achse)
                        for polygon in polygons {
                            let rotation = (0.0, 0.0, 10.0_f32.to_radians());
                            polygon.rotate_around_center(rotation);
                        }
                    }



                    // Skalierung
                    0x5A => { // 'Z' - Vergrößern (x1.1)
                        for polygon in polygons {
                            let scale = (1.1, 1.1, 1.0);
                            polygon.transform_full((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), scale);
                        }
                    }
                    0x58 => { // 'X' - Verkleinern (x0.9)
                        for polygon in polygons {
                            let scale = (0.9, 0.9, 1.0);
                            polygon.transform_full((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), scale);
                        }
                    }

                    _ => (),
                }
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

    if let Some(ref mut polygons) = POLYGONS {

        // Bewegung
        if keys[0x41] { // 'A' - Nach links
            for polygon in polygons.iter_mut() {
                let translation = (-0.1, 0.0, 0.0);
                polygon.transform_full(translation, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
            }
        }
        if keys[0x44] { // 'D' - Nach rechts
            for polygon in polygons.iter_mut() {
                let translation = (0.1, 0.0, 0.0);
                polygon.transform_full(translation, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
            }
        }
        if keys[0x57] { // 'W' - Nach oben
            for polygon in polygons.iter_mut() {
                let translation = (0.0, 0.1, 0.0);
                polygon.transform_full(translation, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
            }
        }
        if keys[0x53] { // 'S' - Nach unten
            for polygon in polygons.iter_mut() {
                let translation = (0.0, -0.1, 0.0);
                polygon.transform_full(translation, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
            }
        }

        // Rotation
        if keys[0x51] { // 'Q' - Drehe um +10° (X-Achse)
            for polygon in polygons.iter_mut() {
                let rotation = (10.0_f32.to_radians(), 0.0, 0.0);
                polygon.rotate_around_center(rotation);
            }
        }
        if keys[0x52] { // 'R' - Drehe um +10° (Y-Achse)
            for polygon in polygons.iter_mut() {
                let rotation = (0.0, 10.0_f32.to_radians(), 0.0);
                polygon.rotate_around_center(rotation);
            }
        }
        if keys[0x45] { // 'E' - Drehe um +10° (Z-Achse)
            for polygon in polygons.iter_mut() {
                let rotation = (0.0, 0.0, 10.0_f32.to_radians());
                polygon.rotate_around_center(rotation);
            }
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

fn render_scene(polygons: &Vec<Polygon>, focal_length: f32, framebuffer: &mut Framebuffer){
    for polygon in polygons {
        let polygon_2d = project_polygon(&polygon, focal_length, framebuffer.width, framebuffer.height);
        framebuffer.draw_polygon(&polygon_2d, polygon.color);
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

        const FOCAL_LENGTH: f32 = 800.0;
        POLYGONS = Some(vec![{
            let mut polygon = Polygon::new(0xFFFFFFFF); // Weißes Polygon
            polygon.add_point(Point::new(-1.0, -1.0, 5.0));
            polygon.add_point(Point::new(1.0, -1.0, 5.0));
            polygon.add_point(Point::new(0.0, 1.0, 5.0));
            polygon
        }]);

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


        let rotation_speed = 0.0174533; // 1 Grad in Radiant (1° = π/180)
        let mut msg: MSG = std::mem::zeroed();

        loop {

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
                    render_scene(polygons, 800.0, &mut framebuffer);
                }
            };

            // Zeichne den Frame
            draw_frame(&framebuffer, WINDOW_WIDTH, WINDOW_HEIGHT, hbitmap, pixels, hdc, window_hdc);

        }

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

    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        for vertex in &mut self.vertices {
            vertex.x += dx;
            vertex.y += dy;
            vertex.z += dz;
        }
    }

    pub fn transform(&mut self, matrix: Matrix4x4) {
        for vertex in &mut self.vertices {
            *vertex = matrix.multiply_point(vertex);
        }
    }

    pub fn rotate_z(&mut self, angle_radians: f32) {
        // Wende die Rotation auf jeden Punkt an
        for vertex in self.vertices.iter_mut() {
            *vertex = rotate_point_around_z(*vertex, angle_radians);
        }
    }

    pub fn transform_full(
        &mut self,
        translation: (f32, f32, f32), // Verschiebung in (x, y, z)
        rotation: (f32, f32, f32),   // Rotation in (x, y, z) [in Radiant]
        scale: (f32, f32, f32),      // Skalierung in (x, y, z)
    ) {
        // Erzeuge die Transformationsmatrizen
        let translation_matrix = Matrix4x4::translate(translation.0, translation.1, translation.2);
        let rotation_x_matrix = Matrix4x4::rotate_x(rotation.0);
        let rotation_y_matrix = Matrix4x4::rotate_y(rotation.1);
        let rotation_z_matrix = Matrix4x4::rotate_z(rotation.2);
        let scaling_matrix = Matrix4x4::scale(scale.0, scale.1, scale.2);

        // Kombiniere alle Matrizen: Skalieren → Rotieren (X → Y → Z) → Verschieben
        let combined_matrix = translation_matrix
            .multiply(&rotation_z_matrix)
            .multiply(&rotation_y_matrix)
            .multiply(&rotation_x_matrix)
            .multiply(&scaling_matrix);

        // Wende die kombinierte Matrix auf jedes Vertex an
        for vertex in &mut self.vertices {
            *vertex = combined_matrix.multiply_point(vertex);
        }
    }
    pub fn rotate_around_center(&mut self, rotation: (f32, f32, f32)) {
        // 1. Berechne den Mittelpunkt des Polygons
        let mut center_x = 0.0;
        let mut center_y = 0.0;
        let mut center_z = 0.0;
        let vertex_count = self.vertices.len() as f32;

        for vertex in &self.vertices {
            center_x += vertex.x;
            center_y += vertex.y;
            center_z += vertex.z;
        }

        center_x /= vertex_count;
        center_y /= vertex_count;
        center_z /= vertex_count;

        // 2. Verschiebe das Polygon relativ zum Ursprung
        let translation_to_origin = (-center_x, -center_y, -center_z);
        self.transform_full(translation_to_origin, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));

        // 3. Führe die Rotation um den Ursprung durch
        self.transform_full((0.0, 0.0, 0.0), rotation, (1.0, 1.0, 1.0));

        // 4. Verschiebe das Polygon zurück an seinen ursprünglichen Ort
        let translation_back = (center_x, center_y, center_z);
        self.transform_full(translation_back, (0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
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

#[derive(Clone)]
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



        fn draw_polygon(&mut self, polygon: &Polygon2D, color: u32) {

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
        let v0 = snap_to_pixel(v0);
        let v1 = snap_to_pixel(v1);
        let v2 = snap_to_pixel(v2);

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

    //Sicher gehen dass eingegebenes Polygon auch ccw ist sonst reversen
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

#[inline(always)]
fn snap_to_pixel(point: Point2D) -> Point2D {
    Point2D {
        x: point.x.round(),
        y: point.y.round(),
        z: point.z, // z kann unangetastet bleiben
    }
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

#[derive(Debug, Clone, Copy)]
pub struct Matrix4x4 {
    pub data: [[f32; 4]; 4],
}

impl Matrix4x4 {
    pub fn identity() -> Self {
        Self {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn multiply_point(&self, point: &Point) -> Point {
        let x = self.data[0][0] * point.x + self.data[0][1] * point.y
            + self.data[0][2] * point.z + self.data[0][3] * 1.0;

        let y = self.data[1][0] * point.x + self.data[1][1] * point.y
            + self.data[1][2] * point.z + self.data[1][3] * 1.0;

        let z = self.data[2][0] * point.x + self.data[2][1] * point.y
            + self.data[2][2] * point.z + self.data[2][3] * 1.0;

        Point::new(x, y, z)
    }

    pub fn multiply(&self, other: &Matrix4x4) -> Matrix4x4 {
        let mut result = Matrix4x4::identity();
        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = (0..4).map(|k| self.data[i][k] * other.data[k][j]).sum();
            }
        }
        result
    }
    pub fn translate(tx: f32, ty: f32, tz: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        matrix.data[0][3] = tx;
        matrix.data[1][3] = ty;
        matrix.data[2][3] = tz;
        matrix
    }

    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        matrix.data[0][0] = sx;
        matrix.data[1][1] = sy;
        matrix.data[2][2] = sz;
        matrix
    }

    pub fn rotate_z(angle: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        matrix.data[0][0] = cos_theta;
        matrix.data[0][1] = -sin_theta;
        matrix.data[1][0] = sin_theta;
        matrix.data[1][1] = cos_theta;
        matrix
    }

    pub fn rotate_x(angle: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        matrix.data[1][1] = cos_theta;
        matrix.data[1][2] = -sin_theta;
        matrix.data[2][1] = sin_theta;
        matrix.data[2][2] = cos_theta;
        matrix
    }

    pub fn rotate_y(angle: f32) -> Self {
        let mut matrix = Matrix4x4::identity();
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        matrix.data[0][0] = cos_theta;
        matrix.data[0][2] = sin_theta;
        matrix.data[2][0] = -sin_theta;
        matrix.data[2][2] = cos_theta;
        matrix
    }

}
