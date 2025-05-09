use image::{GenericImageView, open};

#[derive(Debug, Clone)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>, // RGBA-Werte).
}

impl Texture {
    pub fn from_file(filepath: &str) -> Self {
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
}
