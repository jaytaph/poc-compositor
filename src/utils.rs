use image::ImageReader;
use vello::peniko::{Blob, Image, ImageFormat};

// Load a PNG image from disk and create a Vello Image
pub fn load_image(path: &str) -> Image {
    let img = ImageReader::open(path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image")
        .into_rgba8();

    let (width, height) = img.dimensions();
    let data = img.into_raw();

    Image::new(Blob::from(data), ImageFormat::Rgba8, width, height)
}