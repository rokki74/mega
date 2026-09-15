use ort::session;
use crate::{frame::Frame, model_session};

pub fn open(location: &String)->image::DynamicImage{
    image::open(location).unwrap()
}

pub fn open_image(location: &String)->(Frame, session::Session){
    let img = image::open(location).unwrap();
    let frame = Frame::from_image(img);

    let session = model_session::init_yolo_sesion(true);

    (frame, session)
}


