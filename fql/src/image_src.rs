use image::DynamicImage;
use ort::{value::TensorRef, session::Session};

use crate::{frame::Frame};

pub fn process_image(location: &String)->Result<(), Box<dyn std::error::Error>>{
    let session = Session::builder()?.commit_from_file("/models/yolo11n.onnx")?;
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


