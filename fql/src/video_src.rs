use crate::{frame::Frame,ffm_frame::FfmFrame, opencv_frame::OpenCvFrame, cam_frame::CamFrame};
use crate::model_session;
use ort::{session::Session};


#[derive(Debug)]
pub enum ApiPref{
    OCV,
    FFP,
    CAM,
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

pub enum FrameIter{
    Ocv(OpenCvFrame),
    Ffm(FfmFrame),
    FileSourced(Box<dyn Iterator<Item=Frame>>),
    //FIX ME: later adding CAM(CamFrame)
    Cam(CamFrame),
}

impl Iterator for FrameIter{
    type Item = Frame;
    fn next(&mut self) -> Option<Self::Item> {
       match self{
           FrameIter::Ocv(iter) =>{
               iter.next()
           },
           FrameIter::Ffm(iter) =>{
               iter.next()
           },
           FrameIter::FileSourced(iter) =>{
               iter.next()
          },
          FrameIter::Cam(cm)=>{
              cm.next()
          }
       } 
    }
}

impl VideoSrc{
    pub fn open(url: &String, preference: &ApiPref) ->(FrameIter, Session){
       println!("OPENING VID of url: {}", url);
       
       let session = model_session::init_yolo_sesion(true);

       match preference{
           ApiPref::OCV =>{
               let frms_iter = OpenCvFrame::open(url);
               (FrameIter::Ocv(frms_iter), session)
           },
           ApiPref::FFP =>{
               let frms_iter = FfmFrame::open(url);
               (FrameIter::Ffm(frms_iter), session)
           },
           ApiPref::CAM =>{
               //takes in cam id
               let frms_iter = CamFrame::open(0);
               (FrameIter::Cam(frms_iter), session)
           }
       }
    }
}

