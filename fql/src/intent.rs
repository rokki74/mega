/*
use crate::{Location, frame::Frame, video_src::{ApiPref, VideoSrc}};
use crate::image_src::{open_image};


type SrcFrame = Frame;
type DstFrame = Frame;
type FrameA = Frame;
type FrameB = Frame;


enum QueryType{
   Video(Option<Location>),
   Image(Option<Location>),
}

enum IntentType<'a>{
  Match(&'a Frame, &'a Frame),
  Find(&'a Frame),
  Replace(&'a SrcFrame, &'a DstFrame),
  Append(&'a FrameA, &'a FrameB),
  Delete(&'a Frame),
}

pub struct Intent<'a>{
    query_type: QueryType,
    intent_type: IntentType<'a>,
    resources: Vec<&'a Frame>,
}

impl Intent<'_>{
   fn solve(&self){
     println!("solving intent");

     match &self.query_type{
         QueryType::Video(vid) =>{
             match vid{
                Some(location) =>{
                   VideoSrc::open(location, &ApiPref::OCV);
                },
                None =>{
                   println!("No video found in query")
                }
             }
         },
         QueryType::Image(img)=>{
             match img{
                 Some(location) =>{
                    open_image(location);
                 },
                 None =>{
                    println!("No image found in query")
                 }
             }
         },
     }
   }     

   fn close(&self){
         println!("closing intent..")
   }
}

*/
