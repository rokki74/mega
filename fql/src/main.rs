pub mod detection;
pub mod ffm_frame;
pub mod frame;
pub mod image_src;
pub mod intent;
pub mod opencv_frame;
pub mod video_src;


type FQL = String;
type TextContent = String;
type Location = String;

fn main() {    println!("The file query language, find/search text in images, find files, images/characters in a video, summarize a video etc");



}

fn build_intent(fql: FQL)->intent::Intent<'static>{
  todo!();
}

fn search_text_in_image(text: TextContent)->Vec<String>{
     todo!();
}
