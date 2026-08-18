use std::time::Duration;
use opencv::{core::Mat, videoio::{CAP_ANY, VideoCapture, VideoCaptureTraitConst}};
use crate::frame::Frame;
use opencv::prelude::{VideoCaptureTrait, MatTraitConst};

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

       let frame_number = self.capture.get(opencv::videoio::CAP_PROP_POS_FRAMES).expect("Unable to get the frame number from opencv");
       let time_ms = self.capture.get(opencv::videoio::CAP_PROP_POS_MSEC)
           .expect("Error getting time from opencv");
       let _fps = self.capture.get(opencv::videoio::CAP_PROP_FPS);

       let timestamp = Duration::from_secs((time_ms / 1000.0)as u64);
       Some(Frame::from_opencv(&self.mat, timestamp, frame_number as usize))
    }
}

impl OpenCvFrame{
    pub fn open(url: &String)->OpenCvFrame{
        let mat = Mat::default();
        let capture = VideoCapture::from_file(url, CAP_ANY).unwrap();

        OpenCvFrame { capture, mat}
    }
}
