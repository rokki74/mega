use std::collections::HashSet;
use crate::{frame::Frame,ffm_frame::FfmFrame, opencv_frame::OpenCvFrame};
use crate::model_session;

pub enum ApiPref{
    OCV,
    FFP,
}

pub enum VideoSrc{
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

impl VideoSrc{
    pub fn open_and_process_video(url: &String, preference: ApiPref){
       let mut session = model_session::init_yolo_sesion(true);

       let selections = HashSet::new();
       match preference{
           ApiPref::OCV =>{
               let frms_iter = OpenCvFrame::open(url);
               for frame in frms_iter{
                   frame.process_frame(&mut session, true, &selections);
               }
           },
           ApiPref::FFP =>{
               let frms_iter = FfmFrame::open(url);
               for mut frame in frms_iter{
                   frame.frm_no +=1;
                   frame.process_frame(&mut session, true, &selections);
               }
           }
       }
    }
}

