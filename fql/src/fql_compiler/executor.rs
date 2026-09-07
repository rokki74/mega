use crate::fql_compiler::parser::{Statement, BuildStatement, DeleteStatement, SelectStatement, UpdateStatement, Parser, Object, FqlOutCome};
use std::{collections::HashMap, io::Write, net::TcpStream};


pub struct Executor<'a>{
  parser: Parser<'a>,
  writer: &'a mut TcpStream,
}

impl <'a> Executor<'a>{
    pub fn new(writer: &'a mut TcpStream)->Executor<'a>{
        let _ = writer.write_all(b"__Welcome to FQL__");
         
        Self{
         parser: Parser::empty(),
         writer,
       }
    }

    pub fn execute(&mut self, fql: String){
       let queries = fql.split(";");

       for query in queries{
           let stmt = self.parser.parse(query.to_string());
           let fql_outcm = stmt.execute();

           if let Some(outcome) = fql_outcm{
             self.send_back_result(outcome);
           }else{
               self.writer.write_all(b"Displaying 0 results");
               self.writer.flush();
           }
       }
    }

    pub fn send_back_result(&mut self, outcome: FqlOutCome){
        match outcome{
            FqlOutCome::Single(datum) =>{
                let final_out = "(fql): ".to_string() + &datum;
                let outcm_bytes = final_out.into_bytes();
                let _ = self.writer.write_all(&outcm_bytes);
                let _ = self.writer.flush();

            },
            FqlOutCome::Multiple(multi_data) =>{
                for datum in multi_data{
                     let final_out = "(fql): ".to_string() + &datum;
                     let outcm_bytes = final_out.into_bytes();

                    let _ = self.writer.write_all(&outcm_bytes);
                    let _ = self.writer.flush();
                }
            }
        }
    }
}



impl Statement for DeleteStatement{
    fn execute(&self)->Option<String> {
        todo!()
    }
}

impl Statement for UpdateStatement{
    fn execute(&self)->Option<String> {
        todo!()
    }
}

impl Statement for BuildStatement{
    fn execute(&self)->Option<FqlOutCome> {
        todo!()
    }
}

impl Statement for SelectStatement{
    fn execute(&self) -> Option<FqlOutCome>{
       let (frms_iter, session) = VideoSrc::open(&self.url, &self.preference); 
//chunk_time ???
       
       let table: HashMap<usize, Option<Duration>> = HashMap::new();
       match self.object{
           Object::Image(img_url) =>{
               //find where this image occurs in the video 
               let d_img = image::open(img_url).expect("error opening image");
               let my_image = Frame::from_image(d_img);

               let mut frms_db = FramesDB{source: UrlSrc::Img(img_url), target: None, table};
               //matching logic
               
               println!("FOUND FRAMEID'S: FRAME TIMESTAMP");
               println!("#?{}",frms_db);
               return frms_db;
           },
           Object::Frame =>{
               let mut frms_db = FramesDB{source: UrlSrc::Vid(self.url), table, target: None};
               for frm in frms_iter{
                   frms_db.table.insert(frm.frm_no, frm.timestamp);
               }


               println!("FOUND FRAMEID'S: FRAME TIMESTAMP");
               println!("#?{}",frms_db);
               return frms_db;
           },
       }
    }
}
