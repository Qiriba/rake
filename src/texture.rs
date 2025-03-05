use image::{open, GenericImageView};


#[derive(Debug, Clone)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>, // RGBA-Werte).
}

impl Texture {
    pub fn from_file(filepath: &str) -> Self {
        // Versuche, die Datei zu öffnen
        let img = open(filepath).expect("Fehler beim Laden der Texturdatei!");

        let (width, height) = img.dimensions();

        let img = img.to_rgba8();

        let data = img.into_raw();

        Texture {
            width: width as usize,
            height: height as usize,
            data,
        }
    }

    // Zugriff auf das Pixel mit Texturkoordinaten (u, v)
    pub fn get_pixel(&self, u: f32, v: f32) -> [u8; 4] {
        let x = ((u * self.width as f32) as usize).clamp(0, self.width - 1);
        let y = ((v * self.height as f32) as usize).clamp(0, self.height - 1);

        let index = (y * self.width + x) * 4; // wir nehmen an, dass `data` RGBA speichert
        [
            self.data[index],
            self.data[index + 1],
            self.data[index + 2],
            self.data[index + 3],
        ]
    }
}
