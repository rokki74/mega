use image::{RgbImage};
use ndarray::Array4;

pub struct LetterBoxedFrame {
    pub image: RgbImage,
    pub info: LetterBoxInfo,
}

pub struct LetterBoxInfo {
    pub scale: f32,
    pub pad_x: f32,
    pub pad_y: f32,
}

impl LetterBoxedFrame {
    pub fn to_tensor(&self) -> Result<Array4<f32>, ndarray::ShapeError> {
        let width = self.image.width() as usize;
        let height = self.image.height() as usize;

        let mut tensor = vec![0.0f32; 3 * width * height];

        for y in 0..height {
            for x in 0..width {
                let pixel = self.image.get_pixel(x as u32, y as u32);

                let r = pixel[0] as f32 / 255.0;
                let g = pixel[1] as f32 / 255.0;
                let b = pixel[2] as f32 / 255.0;

                tensor[y * width + x] = r;

                tensor[width * height + y * width + x] = g;

                tensor[2 * width * height + y * width + x] = b;
            }
        }

        Array4::from_shape_vec(
            (1, 3, height, width),
            tensor
        )
    }
}

impl LetterBoxInfo {
    pub fn to_original(
        &self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> (f32, f32, f32, f32) {

        let x1 = (x1 - self.pad_x) / self.scale;
        let y1 = (y1 - self.pad_y) / self.scale;

        let x2 = (x2 - self.pad_x) / self.scale;
        let y2 = (y2 - self.pad_y) / self.scale;

        (x1, y1, x2, y2)
    }
}
