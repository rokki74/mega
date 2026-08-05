use crate::{frame::Frame,ffm_frame::FfmFrame, opencv_frame::OpenCvFrame};

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
    pub fn open(url: &String, preference: ApiPref)->VideoSrc{
        match preference{
            ApiPref::FFP =>{
                let frms = FfmFrame::open(url);
                VideoSrc::FFP(frms)
            },
            ApiPref::OCV =>{
                let frms = OpenCvFrame::open(url);
                VideoSrc::OCV(frms)
            }
        }
    }

    pub fn open_and_process_video(url: &String, preference: ApiPref){
       match preference{
           ApiPref::OCV =>{
               let frms_iter = OpenCvFrame::open(url);
               for frame in frms_iter{
                   frame.process_frame();
               }
           },
           ApiPref::FFP =>{
               let frms_iter = FfmFrame::open(url);
               for frame in frms_iter{
                   frame.process_frame();
               }
           }
       }
    }
}

