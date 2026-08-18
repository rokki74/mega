use std::path::Path;
use ort::{session::Session};

pub fn init_yolo_sesion(use_yolo26: bool)->Session{
         let model_file = if use_yolo26{
                 concat!("/home/nines/Desktop/rusty/mega/fql/", "models/yolo26n.onnx")
             }else{
                 concat!("/home/nines/Desktop/rusty/mega/fql/", "models/yolo11n.onnx")
             };

        let model_path = Path::new(model_file);
        if !model_path.exists(){
            println!("Model path does not exist!");
        }
        
        let session = Session::builder().unwrap().commit_from_file(model_file);
        session.expect("failed to init a yolo model's session!")
}
