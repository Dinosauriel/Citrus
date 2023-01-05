use crate::graphics::buffer;
use image;

pub struct Texture {
    buffer: buffer::Buffer,
}

impl Texture {
    pub fn new_from_bytes(bytes: &[u8], width: u32, height: u32) {
        let img: image::ImageBuffer<image::Rgba<u8>, &[u8]> = image::ImageBuffer::from_raw(width, height, bytes).expect("couldnt read raw image");
        img.save("texture.png").expect("couldnt save image");
    }
}
