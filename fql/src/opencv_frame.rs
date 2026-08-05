use opencv::{core::Mat, videoio::{CAP_ANY, VideoCapture}};
use crate::frame::Frame;

pub struct OpenCvFrame{
    capture: VideoCapture,
    mat: Mat,
}

impl Iterator for OpenCvFrame{
    type Item = Frame;
    fn next(&mut self) -> Option<Self::Item> {
       if !self.capture.read(&mut self.mat).ok()?{
         return None;
       }

       if self.mat.empty(){
         return None;
       }

       Some(Frame::from_opencv(&self.mat))
    }
}

impl OpenCvFrame{
    pub fn open(url: &String)->OpenCvFrame{
        let mat = Mat::default();
        let capture = VideoCapture::from_file(url, CAP_ANY).unwrap();

        OpenCvFrame { capture, mat }
    }
}


