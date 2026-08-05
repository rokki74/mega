use ffmpeg_next::{self as ffmpeg, format::Pixel, software::scaling::flag::Flags};
use image::DynamicImage;
use opencv::{self, core::{Mat, MatTraitConst, MatTraitConstManual}, imgproc};
use ort::{value::TensorRef, session::Session};
use ndarray::Array4;

use crate::{detection::{Detection, iou}};


#[derive(Debug, Clone)]
pub enum PixelFormat{
    Rgb8,
    Rgba8,
    Gray8,
    Bgr8,
}

#[derive(Debug, Clone)]
pub struct Frame{
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    pub data: Vec<u8>,
}

impl Frame{
    pub fn from_image(img: DynamicImage)->Self{
        let rgb = img.to_rgb8();

        Self{
            width: rgb.width() as usize,
            height: rgb.height() as usize,
            format: PixelFormat::Rgb8,
            data: rgb.into_raw(),
        }
    }

    pub fn from_ffmpeg(ffrm: &ffmpeg::frame::Video)->Self{
       let mut scalar = Context::get(
           ffrm.format(),
           ffrm.width(),
           ffrm.height(),
           Pixel::RGB24,
           ffrm.width(),
           ffrm.height(),
           Flags::BILINEAR,
       ).unwrap();

       let mut rgbed = ffmpeg::frame::Video::empty();

       scalar.run(ffrm, &mut rgbed).unwrap();

       let width = rgbed.width() as usize;
       let height = rgbed.height() as usize;

       let rgb = if ffrm.format() == Pixel::RGB24{
           return Self{
               width,
               height,
               format: PixelFormat::Rgba8,
               data: rgbed.data(0).to_vec(),
           }
       };

       let stride = rgbed.stride(0);
       let src = rgbed.data(0);

       let mut out = Vec::with_capacity(width as usize * height as usize * 3);

       for y in 0..height{
           let start = y * stride;
           let end = start + width * 3;

           out.extend_from_slice(&src[start..end]);
       }
       
       Self{
           width,
           height,
           format: PixelFormat::Rgb8,
           data: rgbed.data(0).to_vec(),
       }
    }

    pub fn from_opencv(mat: &Mat)->Self{
       let mut rgb_mat = Mat::default();
       opencv::imgproc::cvt_color(mat, &mut rgb_mat, imgproc::COLOR_BGR2RGB, 0);
      
       let mut rgb = Mat::default();
       Frame::resize_to_640(&rgb);
       let width = rgb_mat.cols() as usize;
       let height = rgb_mat.rows() as usize;
       let format = PixelFormat::Rgb8;

       let data = rgb_mat.data_bytes().expect("Couldn't get the rgb data byte on opencv").to_vec();

       Self{
           width,
           height,
           format,
           data,
       }
    }

    fn resize_to_640(mat: &Mat){

    }

    pub fn process_frame(&self){
        let session =  Session::builder()?.commit_from_file("/models/yolo11n.onnx")?;
        let tensor = self.to_tensor();

        let input = TensorRef::from_array_view(tensor.view());
        let outputs = session.run(ort::inputs![input])?;

        let output = outputs[0].extract_array::<f32>()?;

        println!("output shape: {:?}", output.shape());
        //decoding one prediction
        let pred = output.index_axis(ndarray::Axis(2), 0);
        let cx = pred[[0,0]];
        let cy = pred[[0,1]];
        let w = pred[[0,2]];
        let h = pred[[0,3]];
        
        //4..84 is class scores
        //finding the best class
        let mut best_class = 0;
        let mut best_score = 0.0;

        for c in 0..80{
            let score = pred[[0, 4+c]];
            if score > best_score{
               best_score = score;
               best_class = c;
            }
        }
        
        if best_score >0.5{
            println!("high confidence score!")
        }

        //converting center into rectangle coords
        let x1 = cx -w /2.0;
        let y1 = cy -h /2.0;
        let x2 = cx + w/2.0;
        let y2 = cy + h/2.0;

        //scaling back to original img size 
        let scale_x = 1920.0 / 640.0;
        let scale_y = 1080.0 /640.0;

        let x1 = x1 * scale_x;
        let y2 = y2 * scale_y;
        let x2 = x2 * scale_x;
        let y2 = y2 * scale_x;
        //now the ox has regained original frame oordinates
        
        let detections = Vec::new();
        detections.push(Detection{
            class_id: best_class,
            score: best_score,
            x1,
            x2,
            y1,
            y2,
        });

        //nms to remove duplicates, Non-Maximum Suppression(NMS)
        //sorts in descending order.
        // detections.sort_by(|a, b| b.score.total_cmp(&a.score));
        
        detections.sort_by(|a, b| b.score.total_cmp(&a.score));
        let mut final_dets = vec::new();
        while let Some(mut det) = detections.pop(){
            final_dets.push(det);

            detections.retain(|other|{
                //intersection over union
                iou(&det, *other) < 0.45
            });
        }
    }

    pub fn to_tensor(&self)->Array4<f32>{
       let (width, height, data) = (self.width, self.height, self.data);
       let mut tensor = vec![0f32;3 * width * height];

       for y in 0..height{
           for x in 0..width{

               let pixel = (y * width + x) * 3;
               
               let r = data[pixel] as f32 / 255.0;
               let g = data[pixel + 1] as f32 / 255.0;
               let b = data[pixel + 2] as f32 / 255.0;

               tensor[y * width + x] = r;
               tensor[width * height + y * width + x] = g;
               tensor[2 * width * height + y * width + x] = b;
           }
       }
       
       let arr = Array4::from_shape_vec((1, 3, width, height), data);
       
       arr
    }
}


