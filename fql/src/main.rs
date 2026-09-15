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

use std::{io::{BufReader, BufRead, Write}, net::{TcpListener, TcpStream}, thread};
use crate::{fql_compiler::executor::Executor};

type FQL = String;
type TextContent = String;
type Location = String;

fn main() {    
   let listener = TcpListener::bind("127.0.0.1:5656").unwrap();

   for stream in listener.incoming(){
      match stream{
          Ok(stream)=>{
              thread::spawn(||{
                handle_conn(stream);    
              });
          },
          Err(err)=>{
              eprintln!("Error occurred listening to the stream.  Err: {}", err);
          }
      } 
   }
}

fn handle_conn(stream: TcpStream){
   let mut writer = match stream.try_clone(){
       Ok(s) => s,
       Err(e)=>{
          eprintln!("Error creating a writer(clone) from stream, Error: {}", e); 
          return
       }
   };
   
   let mut executor = Executor::new(&mut writer);

   let mut head_starter = match stream.try_clone(){
       Ok(s) => s,
       Err(e) => {
           eprint!("Error creating head_starter clone from stream. Error: {}", e);
           return;
       },
   };

   let mut reader = BufReader::new(stream);

   let mut input_line = String::new();
   let head_start = b"(fql)> ";
   let _ = head_starter.write_all(head_start);
   while let Ok(bytes_read) = reader.read_line(&mut input_line){
      if bytes_read == 0{
          println!("User exited early");
          return;
      }

      let fql_query = input_line.trim();

      if fql_query == "exit"{
          println!("Exit..");
          break;
      }
      let _fql_outcome = executor.execute(fql_query.to_string());


       let _ = head_starter.write_all(head_start);
      }
}

/*
fn cli_mode(args: env::args){
    println!("The file query language, find/search text in images, find files, images/characters in a video, summarize a video etc");
  

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
} */
