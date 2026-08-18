use crate::{frame::Frame, model_session};

pub fn process_image(location: &String){
    let img = image::open(location).unwrap();
    let frame = Frame::from_image(img);

    let mut session = model_session::init_yolo_sesion(true);
    frame.process_frame(&mut session, true);
}


