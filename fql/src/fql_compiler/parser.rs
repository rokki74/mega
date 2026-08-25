use std::{collections::HashMap, hash::{BuildHasher, Hash}, time::Duration};
use crate::{fql_compiler::lexer::{Lexer, Token, TokenType}, frame::Frame, image_src, video_src::{ApiPref, VideoSrc}};

struct Parser<'a>{
    keywords: &'a HashMap<&'a str, TokenType>,
    lexer: &'a Lexer,
    peek_token: Token,
    cur_token: Token,
}

enum Object {
    Frame,
    Image(String),
}

struct SelectStatement{
    object: Object,
    url: String,
    timeline: Option<(Duration, Duration)>,
    expr: Option<String>,
    preference: ApiPref,
}

impl SelectStatement{
    fn empty()->SelectStatement{
        SelectStatement { 
            object: Object::Frame,
            url: String::new(),
            timeline: None,
            expr: None,
            preference: ApiPref::OCV,
        }
    }
}

impl <'a> Parser<'a>{
    fn empty()->Parser<'a>{
        Parser{
            keywords: &HashMap::new(),
            lexer: &Lexer::empty(),
            peek_token: Token::empty(),
            cur_token: Token::empty(),
        }
    }
}

trait Statement{
  fn execute(&self){}
}

#[derive(Debug)]
enum UrlSrc{
    Vid(String),
    Img(String),
}

#[derive(Debug)]
struct FramesDB{
    source: UrlSrc,
    target: Option<UrlSrc>,
    table: HashMap<usize, Option<Duration>>,
}

struct DeleteStatement{
   target: Option<TargetObj>,
   non_targetted: Option<Vec<usize>>,
}

impl DeleteStatement{
   pub fn empty()->DeleteStatement{
      DeleteStatement { target: None,
      non_targetted: None,
      }
  }

   fn execute(&self)->Result<>{


       Ok()
   }
}

type FrameId = usize;
type URL = String;
enum Obj{
    VID(URL),
    IMAGE(URL),
}

type SrcObj = Obj;
type TargetObj = Obj;

enum UpdateObj {
    Frame(FrmSrc),
    Image(URL),
}

enum FrmSrc{
    Frame(FrameId),
    ALL,
}

struct UpdateStatement{
    source: Option<UpdateObj>,
    final_set: Option<UpdateObj>,
}

impl  UpdateStatement {
    fn empty()->UpdateStatement{
        UpdateStatement { source: None, final_set: None }
    }

    fn execute(&self)->Result<>{


       Ok()
   }
}

struct BuildStatement{
   video_name: String,
}

impl BuildStatement{
    fn empty()->BuildStatement{
        BuildStatement { video_name: String::new() }
    }

    fn execute(&self)->Result<>{
        let name = self.video_name.clone();
        println!("Building your video: {}...  ....", name);

        Ok()
    }
}

impl SelectStatement{
    fn execute(&self) -> FramesDB{
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

impl <'a> Parser<'a>{
     fn new()->Parser<'a>{  
       let mut keywords: HashMap<&'a str, TokenType> = HashMap::with_capacity(27_usize);
       
       keywords.insert("illegal", TokenType::Illegal);
       keywords.insert("insert", TokenType::Insert);
       keywords.insert("string", TokenType::String);
       keywords.insert("star", TokenType::Star);
       keywords.insert("number", TokenType::Number);
       keywords.insert("semicolon", TokenType::Semicolon);
       keywords.insert("eq", TokenType::EQ);
       keywords.insert("eof", TokenType::EOF);
       keywords.insert("range", TokenType::Range);
       keywords.insert("objectimage", TokenType::ObjectImage);
       keywords.insert("frameid", TokenType::FrameId);
       keywords.insert("build", TokenType::Build);
       keywords.insert("before", TokenType::Before);
       keywords.insert("and", TokenType::And);
       keywords.insert("after", TokenType::After);
       keywords.insert("where", TokenType::Where);
       keywords.insert("classid", TokenType::ClassId);
       keywords.insert("classname", TokenType::ClassName);
       keywords.insert("video", TokenType::Video);
       keywords.insert("select", TokenType::Select);
       keywords.insert("or", TokenType::Or);
       keywords.insert("into", TokenType::Into);
       keywords.insert("set", TokenType::Set);
       keywords.insert("from", TokenType::From);
       keywords.insert("snapshot", TokenType::Snapshot);
       keywords.insert("update", TokenType::Update);
       keywords.insert("delete", TokenType::Delete);

       let mut parser = Parser::empty();
       parser.keywords = &keywords;

       parser
     }


     pub fn parse_and_execute(&mut self, fql: String)->Option<>{
         self.lexer.new(&fql);
             let token = self.lexer.next_token(self.keywords);
             
             match token.token_type{
                 TokenType::Update => { 
                    let stmt = self.parse_update();
                    let results = stmt.execute();

                    Some(results)
                 },
                 TokenType::Delete => {
                     let stmt = self.parse_delete();
                     let results = stmt.execute();

                     Some(results)
                 },
                 TokenType::Select => {
                     let stmt = self.parse_select(); 
                     let results = stmt.execute();
                     Some(results)
                 },
                 TokenType::Build => {
                     let stmt = self.parse_build();
                     let results = stmt.execute();
                     Some(results)
                 }
                 _=>{ 
                     println!("Illegal value {} in statement: {}", token.value, fql);
                     None
                 },
             }
     }

     fn parse_build(&mut self)->BuildStatement{
          let stmt = BuildStatement::empty();

          self.expect("build");
          stmt.video_name = self.cur_token.value.clone;

          stmt
     }

     fn parse_delete(&mut self) ->DeleteStatement{
         let mut stmt = DeleteStatement::empty();

         self.expect("delete");

         //delete * From video 'url' where FrameId = ''
         match self.cur_token.token_type{
             TokenType::Star =>{
                self.frm_src = Some(FrmSrc::ALL);
             },
             TokenType::FrameId =>{
                 self.expect("frameid");

                 let frm_id = self.cur_token.value;

                 stmt.frm_src = Some(FrmSrc::Frame(frm_id));
             },
             _=>{},
         }
         self.expect("from");
         match self.cur_token.token_type{
             TokenType::Video => {
               self.expect("video"); 

            stmt.target = Some(TargetObj::VID(self.cur_token.value.clone()));
               self.expect("string");
               stmt
             },
             TokenType::ObjectImage =>{
                self.expect("ObjectImage");
                let img_url = self.cur_token.value.clone();

                stmt.target = Some(TargetObj::IMAGE(img_url));

                stmt.non_targetted = None;
                stmt
             },
             _=>{stmt.target = None;
                 stmt 
             }
         }
     }

     fn parse_update(&mut self)->UpdateStatement{
        let stmt = UpdateStatement::empty();
        
        self.expect("update"); 
        match self.cur_token.token_type{
           TokenType::FrameId =>{
               self.expect("frameid");
               let frm_id = self.cur_token.value;
               stmt.source = Some(UpdateObj::Frame(FrmSrc::Frame(frm_id)));

                let _x = self.lexer.next_token(self.keywords);
           },
           TokenType::Star =>{
               self.expect("star");

               stmt.source = Some(UpdateObj::Frame(FrmSrc::ALL));
           },
           _=> {stmt.source = None},
        }

        self.expect("set"); 
        
        match self.cur_token.token_type{
            TokenType::FrameId =>{
               self.expect("frameid");
               let frm_id = self.cur_token.value; 
               stmt.final_set = Some(UpdateObj::Frame(FrmSrc::Frame(frm_id)));

               let _x = self.lexer.next_token(self.keywords);
            },
            TokenType::ObjectImage =>{
               self.expect("objectimage");

               let img_url = self.cur_token.value;
               stmt.final_set = Some(UpdateObj::Image(img_url));
               self.expect("string");
            },
            _=>{stmt.final_set = None}
        }

        stmt
     }

     fn parse_select(self)-> &'a SelectStatement{
        self.expect("select");
       
        let mut stmt = &SelectStatement::empty();
        match self.cur_token.token_type{
           TokenType::Star =>{
               stmt.object = Object::Frame;
               self.expect("from");

               self.expect("video");
               stmt.url = self.cur_token.value;
               
               
               loop{
                  let tok = self.lexer.next_token(&self.keywords);
                  match tok.token_type{
                    TokenType::Range =>{
                       self.expect("range");

                       let start = Duration::from(&self.cur_token.value);

                       self.expect("string");
                       let end = Duration::from(&self.cur_token.value);
                       self.expect("string");

                       stmt.timeline = Some((start, end));
                    },
                    TokenType::Where =>{
                      self.expect("where");

                      stmt.expr = Some(self.lexer.next_token(self.keywords));
                    },
                    _=>{break;},
                  }
               }
           },
           TokenType::ObjectImage =>{
               self.expect("ObjectImage");

               let img_url = self.cur_token.value;
               println!("gotten image url: {}", img_url);
               stmt.object = Object::Image(img_url);
               self.expect("From");
               
               stmt.url = self.cur_token.value;
               println!("gotten video url: {}", stmt.url);
               loop{
                  let tok = self.lexer.next_token(&self.keywords);
                  match tok.token_type{
                    TokenType::Range =>{
                       self.expect("range");

                       let start = self.cur_token.value;
                       let end = self.lexer.next_token(&self.keywords);

                       stmt.timeline = Some((start, end));
                    },
                    TokenType::Where =>{
                      self.expect("where");

                      stmt.expr = Some(self.lexer.next_token(self.keywords));
                    },
                    _=>{break;},
                  }
               }
           },
           _ => {break;},
        } 

        stmt
     }

     fn expect(&self, t: &str){
         if let Some(val) = self.keywords.get(&t){
             println!("Done expecting token: {}, found token type: {}",t, val);

             self.cur_token = self.lexer.next_token(self.keywords);
         }else{
             eprintln!("Error occurred:  unexpected {} in the fql statement", t);
         }
     }
}


