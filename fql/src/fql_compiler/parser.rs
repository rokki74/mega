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
}

type FrameId = usize;
type URL = String;
enum Obj{
    VID(URL),
    IMAGE(URL),
}

type SrcObj = Obj;
type TargetObj = Obj;

struct UpdateStatement{
    source: Option<SrcObj>,
    target: Option<TargetObj>,
}

struct BuildStatement{
   video_name: String,
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


     pub fn parse(self, fql: String){
         self.lexer.new(&fql);
         loop{
             let token = self.lexer.next_token(self.keywords);
             
             match token.token_type{
                 TokenType::Update => self.parse_update(),
                 TokenType::Delete => self.parse_delete(),
                 TokenType::Select => self.parse_select(),
                 TokenType::Build => self.parse_build(),
                 _=>{ println!("Illegal value {} in statement: {}", token.value, fql); break;},
             }
         }
     }

     fn parse_build(&mut self)->BuildStatement{
          let stmt = BuildStatement::empty();

          self.expect("build");
          stmt.video_name = self.cur_token.value;

          stmt
     }

     fn parse_delete(&mut self) ->DeleteStatement{
         let mut stmt = DeleteStatement::empty();

         self.expect("delete");

         //delete * From video 'url' where FrameId = ''
         self.expect("*");
         self.expect("from");
         match self.cur_token.token_type{
             TokenType::Video => {
               self.expect("video"); 

            stmt.target = Some(TargetObj::VID(self.cur_token.value.clone()));
              stmt
             },
             TokenType::ObjectImage =>{
                self.expect("ObjectImage");
                let img_url = self.cur_token.value.clone();

                stmt.target = Some(TargetObj::IMAGE(img_url));

                stmt.non_targetted = None;
                }else{

                   let v = Vec::new();
                   self.expect(strin)
                   loop{
                             
                   } 
                }

                stmt
             },
             _=>{stmt.target = None;
                 stmt 
             }
         }
     }

     fn parse_update(&mut self)->UpdateStatement{
        let stmt = UpdateStatement::empty();
        
        
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


