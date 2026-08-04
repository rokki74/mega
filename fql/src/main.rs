use anyhow::Context;
use ffmpeg_next::{self as ffmpeg, format::{self, Pixel, context}, software::scaling::flag::Flags};
use image::DynamicImage;
use ndarray::Array4;
use ort::{value::TensorRef, Session};
use rustcv::videoio::VideoCapture;
use opencv::{self, core::{Mat, MatTraitConst, MatTraitConstManual}, imgproc, videoio::{self, VideoCaptureTrait, VideoCaptureTraitConst}};

type FQL = String;
type TextContent = String;
type Location = String;

enum QueryType{
   Video(Option<Location>),
   Image(Option<Location>),
}

#[derive(Debug, Clone)]
enum PixelFormat{
    Rgb8,
    Rgba8,
    Gray8,
    Bgr8,
}

#[derive(Debug, Clone)]
struct Frame{
    width: usize,
    height: usize,
    format: PixelFormat,
    data: Vec<u8>,
}

enum VideoSrc{
    OCV(OpenCvFrame),
    FFP(FfmFrame),
}

impl Iterator for VideoSrc{
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        match self{
            VideoSrc::OCV(src) => src.next(),
            VideoSrc::FFP(src) => src.next(),
        }
    }
}

struct FfmFrame{
    ictx: format::Input,
    decoder: ffmpeg::codec::decoder::video,
    frame: ffmpeg::frame::Video,
}

impl Iterator for FfmFrame{
    type Item = Frame;
    fn next(&mut self) -> Option<Self::Item> {
           
    }
}

struct OpenCvFrame{
    capture: VideoCapture,
    mat: Mat,
}

impl Iterator for OpenCvFrame{
    type Item = Frame;
    fn next(&mut self) -> Option<Self::Item> {
       if !self.capture.read(&mut ocv_frame.mat).ok()?{
         return None;
       }

       if self.mat.empty(){
         return None;
       }

       Some(Frame::from_opencv(&ocv_frame.mat))
    }
}

impl Frame{
    fn from_image(img: DynamicImage)->Self{
        let rgb = img.to_rgb8();

        Self{
            width: rgb.width() as usize,
            height: rgb.height() as usize,
            format: PixelFormat::Rgb8,
            data: rgb.into_raw(),
        }
    }

    fn from_ffmpeg(ffrm: ffmpeg::frame::video)->Self{
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

    fn from_opencv(mat: &Mat)->Self{
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

    fn process_frame(&self){
        let session = build_session();
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
        while let mut Some(det) = detections.pop(){
            final_dets.push(det);

            detections.retain(|other|{
                //intersection over union
                iou(&det, *other) < 0.45
            });
        }
    }

    fn to_tensor(&self)->Array4<f32>{
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

fn build_session()->'static &Session{
    &Session::builder()?
        .commit_from_file("/models/yolo11n.onnx")?
}

enum IntentType<'a>{
  Match(&'a Frame, &'a Frame),
  Find(&'a Frame),
}

struct Intent<'a>{
    query_type: QueryType,
    intent_type: IntentType<'a>,
    resources: Vec<&'a Frame>,
}

impl Intent<'_>{
   fn solve(&self){
     println!("solving intent");

     match &self.query_type{
         QueryType::Video(vid) =>{
             match vid{
                Some(location) =>{
                   process_video(location);
                },
                None =>{
                   println!("No video found in query")
                }
             }
         },
         QueryType::Image(img)=>{
             match img{
                 Some(location) =>{
                    process_image(location);
                 },
                 None =>{
                    println!("No image found in query")
                 }
             }
         },
     }
   }     

   fn close(&self){
         println!("closing intent..")
   }
}

fn process_video(url: &String){
    return process_with_rustcv(rvfrm);
}

fn process_with_opencv(url: &String)-> impl Iterator<Item = Frame>{
    let mut capture = videoio::VideoCapture::from_file(url, videoio::CAP_ANY)?;

    if !capture.is_opened()?{
        panic!("video couldn't open with opencv");
    }

    println!("video opened by opencv successfully!");
    
    let frames = Vec::new();

    let mut mat = Mat::default();
    while capture.read(&mut mat).expect("unable to read capture"){
        if mat.empty(){
            break;
        }

        let frame = Frame::from_opencv(&mat);

        frames.push(frame);
    }//capture.read
    
    for frame in frames{
        frame.process_frame();
    }
}



impl FfmFrame{
  fn new(url: &String) ->FfmFrame{
    let mut ictx = ffmpeg::format::input(&url).unwrap();

    let input = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .unwrap();

    let video_stream_index = input.index();

    let context = ffmpeg::codec::context::Context::from_parameters(input.parameters());

    let mut decoder = context.unwrap().decoder().video();
    let mut frame = ffmpeg::frame::Video::empty();

    FfmFrame { ictx, decoder, frame}
  }
}

fn stream_ffmpeg{
    for (stream, packet) in ictx.packets(){
        if stream.index() != video_stream_index{continue;}
        decoder.send_packet(&packet);
    }

    while decoder.receive_frame(&mut frame).is_ok(){
        let my_frame = Frame::from_ffmpeg(&frame);
        my_frame.process_frame(); 
    }
}

fn iou(det: &Detection, other: Detection) ->f32{
    let d_width = det.x2 - det.x1;
    let d_height = det.y2 - det.y1;
    let (d_left, d_right, d_top, d_bottom) =(det.x, (det.x + d_width), det.y, (det.y + d_height));

   let o_width = other.x2 - other.x1;
   let o_height = other.y2 - other.y1;
   let (o_left, o_right, o_top, o_bottom) =(other.x, (other.x + o_width), other.y, (other.y + o_height));

   //unions 
   let left = d_left.max(o_left);
   let right = d_right.min(o_right);
   let top = d_top.max(o_top);
   let bottom = d_bottom.min(o_bottom);

   if left <= right || bottom <= top{
       return 0.0;
   }
   
   let w = right - left;
   let h = bottom - top;

   let intersection = w * h;

   let area_d = d_width * d_height;
   let area_o = o_width * o_height;

   let union = area_d + area_o - intersection;
   intersection/union
}

#[derive(Debug)]
struct Detection{
    class_id: usize,
    score: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

fn detect(mut detections: &Vec<Detection>){
    todo!();
}

fn process_image(location: &String)->Result<(), Box<dyn std::error::Error>>{
    let session = build_session();
    
    let img = image::open(location);
    match img{
        DynamicImage(dyimg) =>{
        let frame = Frame::from_image(dyimg);

        let tensor = frame.to_tensor();

        let input = TensorRef::from_array_view(tensor.view())?;
        let outputs = session.run(ort::inputs![input])?;

        let output = outputs[0].extract_array::<f32>()?;

        println!("output shape: {:?}", output.shape());
        },
        _ =>panic!("Error getting dynamic image"),
    };
}

fn main() {    println!("The file query language, find/search text in images, find files, images/characters in a video, summarize a video etc");



}

fn build_intent(fql: FQL)->Intent{
  todo!();
}

fn search_text(text: TextContent)->vec<String>{
     todo!();
}
