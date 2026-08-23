pub mod detection;
pub mod ffm_frame;
pub mod frame;
pub mod image_src;
pub mod intent;
pub mod opencv_frame;
pub mod video_src;
pub mod fql_compiler;
pub mod letterbox;
pub mod model_session;
pub mod tracker;


use std::{env, path::Path};

use crate::video_src::VideoSrc;

type FQL = String;
type TextContent = String;
type Location = String;

fn main() {    
    println!("The file query language, find/search text in images, find files, images/characters in a video, summarize a video etc");
  
    let args = env::args();

    for file_src in args.skip(1){
      let file_path = Path::new(&file_src);
      if file_path.exists(){
         VideoSrc::open_and_process_video(&file_src, video_src::ApiPref::FFP);
      }else{
          println!("Video file provided: {} doesn't exist", file_src);
      }
    }

}

fn build_intent(_fql: FQL)->intent::Intent<'static>{
  todo!();
}

fn search_text_in_image(text: TextContent)->Vec<String>{
   println!("Received for text search: {}", text);
   vec![text]
}
