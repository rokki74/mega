use std::time::Duration;
use crate::letterbox::{LetterBoxedFrame, LetterBoxInfo};
use ffmpeg_next::{self as ffmpeg, format::Pixel, software::scaling::{flag::Flags, context::Context}};
use image::{DynamicImage, RgbImage};
use opencv::{self, core::{Mat, MatTraitConst, MatTraitConstManual}, imgproc};
use ort::{value::TensorRef, session::Session};
use ndarray::Array4;
use crate::{detection::{Detection, iou, detect}};


#[derive(Debug, Clone)]
pub enum PixelFormat{
    Rgb8,
    Rgba8,
    Gray8,
    Bgr8,
}

#[derive(Debug, Clone)]
pub struct Frame{
    pub frm_no: usize,
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    pub data: Vec<u8>,
    pub timestamp: Option<Duration>,
}

impl Frame{
    pub fn from_image(img: DynamicImage)->Self{
        let rgb = img.to_rgb8();
        
        Self{
            frm_no: 0,
            width: rgb.width() as usize,
            height: rgb.height() as usize,
            format: PixelFormat::Rgb8,
            data: rgb.into_raw(),
            timestamp: None,
        }
    }

    pub fn from_ffmpeg(ffrm: &ffmpeg::frame::Video, time_base: ffmpeg_next::Rational)->Self{
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

        let timestamp_seconds = ffrm.pts().map(|pts| {
        let seconds =
            pts as f64
            * time_base.numerator() as f64
            / time_base.denominator() as f64;

        Duration::from_secs_f64(seconds)
    });

       let timestamp_seconds = timestamp_seconds.expect("Unable to convert timestamp into seconds for frame.");
       if ffrm.format() == Pixel::RGB24{
           return Self{
               frm_no: 0,
               width,
               height,
               format: PixelFormat::Rgba8,
               data: rgbed.data(0).to_vec(),
               timestamp: Some(timestamp_seconds),
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
           frm_no: 0,
           width,
           height,
           format: PixelFormat::Rgb8,
           data: out,
           timestamp: Some(timestamp_seconds),
       }
    }

    pub fn from_opencv(mat: &Mat, timestamp: Duration, frm_no: usize)->Self{
       let mut rgb_mat = Mat::default();
       let _ = opencv::imgproc::cvt_color(mat, &mut rgb_mat, imgproc::COLOR_BGR2RGB, 0);
      
       let width = rgb_mat.cols() as usize;
       let height = rgb_mat.rows() as usize;
       let format = PixelFormat::Rgb8;

       let data = rgb_mat.data_bytes().expect("Couldn't get the rgb data byte on opencv").to_vec();

       Self{
           frm_no,
           width,
           height,
           format,
           data,
           timestamp: Some(timestamp),
       }
    }

    pub fn process_frame(&self, session: &mut Session, use_yolo26:bool){
        let letterboxed = self.letterbox(640);
        let tensor = letterboxed.to_tensor();

        let binding = tensor.expect("failed to successfully get tensor.view()");
        let input = TensorRef::from_array_view(binding.view()).expect("Failed to create TensorRef");

        let outputs = session.run(ort::inputs![input]).unwrap();

        let output = outputs[0].try_extract_array::<f32>().expect("Failure to get output by extractin array at outputs[0] index 0");

        println!("output shape: {:?}", output);

        let num_predictions = output.shape()[1];
        println!("predictions: {}", num_predictions);

        let output = output.index_axis(ndarray::Axis(0), 0);
        println!("Output2's shape: {:?}", output.shape());

        let mut detections = Vec::new();
        if use_yolo26{
            println!("Using yolo26n..");
            for pred in output.axis_iter(ndarray::Axis(0)) {
                let score = pred[4];
                if score < 0.5 {
                    continue;
                }

                println!("CURRENT pred: {}", pred);
                let x1 = pred[0];
                let y1 = pred[1];
                let x2 = pred[2];
                let y2 = pred[3];

                let class_id = pred[5] as usize;


                     
                let (x1, y1, x2, y2) = letterboxed.info.to_original(
                        x1,
                        y1,
                        x2,
                        y2,
                    );

                detections.push(Detection {
                    class_id,
                    score,
                    x1,
                    y1,
                    x2,
                    y2,
                    frm_no: self.frm_no,
                    timestamp: self.timestamp,
                });


            }
        }else{
            println!("Using yolo11n..");
            for pred in output.axis_iter(ndarray::Axis(2)){
                //decoding one prediction
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
                }else{
                    println!("low confidence prediction!");
                    continue;
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
                let y1 = y1 * scale_y;
                let x2 = x2 * scale_x;
                let y2 = y2 * scale_x;
                //now the ox has regained original frame oordinates
                
                detections.push(Detection{
                    class_id: best_class,
                    score: best_score,
                    x1,
                    x2,
                    y1,
                    y2,
                    frm_no: self.frm_no,
                    timestamp: self.timestamp,
                });
            }
        }

        //nms to remove duplicates, Non-Maximum Suppression(NMS)
        //sorts in descending order.
        // detections.sort_by(|a, b| b.score.total_cmp(&a.score));
        
        detections.sort_by(|a, b| a.score.total_cmp(&b.score));
        let mut final_dets = Vec::new();
        while let Some(det) = detections.pop(){
            final_dets.push(det.clone());

            detections.retain(|other|{
                //intersection over union
                iou(&det, other.clone()) < 0.45
            });
        }

        detect(&final_dets); 
    }

    pub fn to_tensor(&self)->Result<Array4<f32>, ndarray::ShapeError>{
       let (width, height) = (self.width, self.height);
       let mut tensor = vec![0f32;3 * width * height];

       for y in 0..height{
           for x in 0..width{

               let pixel = (y * width + x) * 3;
               
               let r = self.data[pixel] as f32 / 255.0;
               let g = self.data[pixel + 1] as f32 / 255.0;
               let b = self.data[pixel + 2] as f32 / 255.0;

               tensor[y * width + x] = r;
               tensor[width * height + y * width + x] = g;
               tensor[2 * width * height + y * width + x] = b;
           }
       }
       
       Array4::from_shape_vec((1, 3, height, width), tensor)
    }

    pub fn letterbox(&self, target: u32)->LetterBoxedFrame{
        let src_width = self.width as u32;
        let src_height = self.height as u32;

        let scale = (target as f32 / src_width as f32)
            .min(target as f32 /src_height as f32);

        let new_width = (src_width as f32 * scale).round() as u32;
        let new_height = (src_height as f32 * scale).round()as u32;
      
        let image = RgbImage::from_raw(src_width, 
            src_height, self.data.clone()
        ).expect("Invalid RGB frame dimensions");

        let resized = image::imageops::resize(
            &image,
            new_width, 
            new_height,
            image::imageops::FilterType::Triangle,);

        let pad_x = (target - new_width)/2;
        let pad_y = (target - new_height)/2;

        let mut canvas = RgbImage::new(target, target);

        image::imageops::overlay(
            &mut canvas,
            &resized,
            pad_x.into(),
            pad_y.into());

        LetterBoxedFrame{
            image: canvas,
            info: LetterBoxInfo{
                scale,
                pad_x: pad_x as f32,
                pad_y: pad_y as f32,
            },
        } 
    }
}


