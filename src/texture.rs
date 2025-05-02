use image::{open, GenericImageView};


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

    // Get texture ID for caching
    pub fn id(&self) -> usize {
        // Use the memory address as a unique identifier
        self as *const Self as usize
    }

    // Convert texture data to u8 slice for SDL
    pub fn data_as_u8_slice(&self) -> &[u8] {
        // Convert from u32 to u8 slice - this assumes texture.data is a Vec<u32>
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                self.data.len() * 4
            )
        }
    }

    // Get texture dimensions
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

}
