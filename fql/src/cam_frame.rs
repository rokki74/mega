use std::time::Duration;
use opencv::{self, core::{Mat, MatTraitConst}, videoio::{CAP_ANY, VideoCapture, VideoCaptureTrait, VideoCaptureTraitConst}};
use crate::frame::Frame;



pub struct CamFrame{
    cam: VideoCapture,
    mat: Mat,
}

impl Iterator for CamFrame{
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cam.read(&mut self.mat).ok()?{
            return None;
        }

        if self.mat.empty(){
            return None;
        }

        let frame_number = self.cam.get(opencv::videoio::CAP_PROP_POS_FRAMES).expect("Unable to get the frame number from opencv");
               let time_ms = self.cam.get(opencv::videoio::CAP_PROP_POS_MSEC)
                   .expect("Error getting time from opencv");
               let _fps = self.cam.get(opencv::videoio::CAP_PROP_FPS);

       let timestamp = Duration::from_secs((time_ms / 1000.0)as u64);
       Some(Frame::from_opencv(&self.mat, timestamp, frame_number as usize))
    }
}

impl CamFrame{
    pub fn open(_cam_id: i32)->Self{
        let mat = Mat::default();
        let cam = VideoCapture::new(0,
            CAP_ANY,
        ).unwrap();


        CamFrame{
           cam,
           mat
        }
    }
}
