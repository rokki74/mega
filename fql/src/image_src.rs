use std::collections::HashSet;

use crate::{frame::Frame, model_session};

pub fn open(location: &String)->image::DynamicImage{
    image::open(location).unwrap()
}

pub fn process_image(location: &String){
    let img = image::open(location).unwrap();
    let frame = Frame::from_image(img);

    let mut session = model_session::init_yolo_sesion(true);
    let selections = HashSet::new();
    frame.process_frame(&mut session, true, &selections);
}


