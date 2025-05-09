use image::{open, GenericImageView};

#[derive(Debug, Clone)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>, // RGBA-Werte).
}

impl Texture {
    pub fn from_file(filepath: &str) -> Result<Self, String> {
        let img = open(filepath);
        if let Ok(img) = img {

            let (width, height) = img.dimensions();

            let img = img.to_rgba8();

            let data = img.into_raw();

            Ok(Texture {
                width: width as usize,
                height: height as usize,
                data,
            })
        }else {
            println!("Failed to load texture from file: {}", filepath);
            Err("Failed to load texture from file ".into())
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
                self.data.as_ptr(),
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
