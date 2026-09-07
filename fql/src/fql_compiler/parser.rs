use std::{collections::HashMap, time::Duration};
use crate::{fql_compiler::lexer::{Lexer, Token, TokenType}, video_src::{ApiPref}};

pub struct Parser<'a>{
    keywords: HashMap<&'a str, TokenType>,
    lexer: Lexer<'a>,
    peek_token: Token,
    cur_token: Token,
}

pub enum Object {
    Frame,
    Image(String),
}

pub enum FqlOutCome{
    Single(String),
    Multiple(Vec<String>),
}

pub trait Statement{
  fn execute(&self)->Option<FqlOutCome>{None}
}

pub enum StatementEnum<'a>{
    Update(UpdateStatement),
    Delete(DeleteStatement),
    Select(&'a SelectStatement),
    Build(BuildStatement),
}

impl <'a>Statement for StatementEnum<'a>{
    pub fn execute()->Option<FqlOutCome>{
       todo!()
    }
}

pub struct SelectStatement{
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
   pub fn empty()->Parser<'a>{
        Parser{
            keywords: HashMap::new(),
            lexer: Lexer::empty(),
            peek_token: Token::empty(),
            cur_token: Token::empty(),
        }
    }
}

#[derive(Debug)]
pub enum UrlSrc{
    Vid(String),
    Img(String),
}

#[derive(Debug)]
pub struct FramesDB{
    source: UrlSrc,
    target: Option<UrlSrc>,
    table: HashMap<usize, Option<Duration>>,
}

pub enum DELETECHOICE{
    IDS(Vec<FrameId>),
    ALL
}

type FrameId = usize;
type URL = String;
enum Obj{
    VID(URL),
    IMAGE(URL),
}

type SrcObj = Obj;
type TargetObj = Obj;
type VidUrl = String;

pub enum UpdateObj{
   ObjectImages(Vec<URL>),
   Vids(Vec<FrameId>, VidUrl)
}


pub struct DeleteStatement{
   target_frames: Option<DELETECHOICE>,
   target_obj: Option<SrcObj>,
}

impl DeleteStatement{
   pub fn empty()->DeleteStatement{
      DeleteStatement { target_frames: None,
      target_obj: None,
      }
  }
}

pub struct UpdateStatement{
    original_obj: Option<UpdateObj>,
    final_obj: Option<UpdateObj>,
}

impl  UpdateStatement {
    fn empty()->UpdateStatement{
        UpdateStatement { original_obj: None, final_obj: None }
    }
}

pub struct BuildStatement{
   video_name: String,
}

impl BuildStatement{
    fn empty()->BuildStatement{
        BuildStatement { video_name: String::new() }
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
       parser.keywords = keywords;

       parser
     }

     pub fn parse(&mut self, query: String)->StatementEnum{
         self.lexer.new(&query);
         let token = self.lexer.next_token(&self.keywords);

         println!("handling the token: token val: {}, token_type: {} Inside the parser", token.value, token.token_type);
             
         match token.token_type{
             TokenType::Update => { 
                self.parse_update()
             },
             TokenType::Delete => {
                 self.parse_delete()
             },
             TokenType::Select => {
                 self.parse_select() 
             },
             TokenType::Build => {
                 self.parse_build()
             }
             _=>{ 
                 println!("Illegal value {} in statement: {}", token.value, query);
                 panic!("Expected an fql statement/query, cannot process a non-fql query input");
             },
         }
     }

     fn parse_build(&mut self)->StatementEnum{
          let mut stmt = BuildStatement::empty();

          self.expect("build");
          stmt.video_name = self.cur_token.value.clone();

          StatementEnum::Build(stmt)
     }

     fn parse_delete(&mut self) ->StatementEnum{
         let mut stmt = DeleteStatement::empty();

         self.expect("delete");

         //delete * From video 'url'
         match self.cur_token.token_type{
             TokenType::Star =>{
                stmt.target_frames = Some(DELETECHOICE::ALL);
             },
             TokenType::FrameId =>{
                 self.expect("frameid");
                 let f_ids:Vec<usize> = Vec::new();
                 loop{
                    let frm_id = self.cur_token.value;
                    
                    let pk_tkn = self.keywords.get(&self.peek_token);
                    if let Some(tkn) = pk_tkn{
                       match tkn{
                           TokenType::Comma =>{
                               f_ids.push(frm_id);
                               self.expect(",");
                           },
                           _ => {
                               f_ids.push(frm_id);
                               break;
                           },
                       }
                    }else{
                        break;
                    }
                 }

                 stmt.target_frames = Some(DELETECHOICE::IDS(f_ids));
             },
             _=>{},
         }
         self.expect("from");
         match self.cur_token.token_type{
             TokenType::Video => {
               self.expect("video"); 

               stmt.target_obj = Some(SrcObj::VID(self.cur_token.value.clone()));
               self.expect("string");
               
               StatementEnum::Delete(stmt) 
             },
             TokenType::ObjectImage =>{
                self.expect("ObjectImage");
                let img_url = self.cur_token.value.clone();

                stmt.target_obj = Some(SrcObj::IMAGE(img_url));

                
                 StatementEnum::Delete(stmt) 
             },
             _=>{stmt.target_obj = None;
                 StatementEnum::Delete(stmt) 
             }
         }
     }

     fn parse_update(&mut self)->StatementEnum{
        let stmt = UpdateStatement::empty();
        
        //Update FrameId '' From Video 'url' Set ObjectImage 'url' / FrameId Video '';
        self.expect("update"); 
        match self.cur_token.token_type{
           TokenType::FrameId =>{
               self.expect("frameid");
               let u_orgn: Vec<FrameId> = Vec::new();
         
               while self.match_next(TokenType::Comma){
                   let frm_id = self.cur_token.value;
                   u_orgn.push(frm_id);
               }
                println!("breaking out no more commas at update statement");

               self.expect("from");
               self.expect("video");
               let url = self.cur_token.value;

               stmt.original_obj = Some(UpdateObj::Vids(u_orgn, url));
           },
           _=> {
                eprintln!("Expected FrameId token found: {} of type: {}",self.cur_token.value, self.cur_token.token_type);
                stmt.original_obj = None;    
           },
        }

        self.expect("set"); 
        match self.cur_token.token_type{
           TokenType::FrameId =>{
               self.expect("frameid");
               let u_fnl: Vec<FrameId> = Vec::new();
         
               while self.match_next(TokenType::Comma){
                   let frm_id = self.cur_token.value;
                   u_fnl.push(frm_id);
               }
                println!("breaking out no more commas on update statement");

               self.expect("from");
               self.expect("video");
               let url = self.cur_token.value;

               stmt.original_obj = Some(UpdateObj::Vids(u_fnl, url));
           },
           TokenType::ObjectImage =>{
               self.expect("objectimage");

               let ob_imgs: Vec<URL> = Vec::new();

               loop{
                   let url = self.cur_token.value;

                   ob_imgs.push(url);

                   match self.peek_token.token_type{
                       TokenType::Comma=>{
                          self.expect(",");
                        },
                        _=>{
                            println!("No more commas!");
                            break;
                        },
                   }
                }
           },
           _=> {
                eprintln!("Expected FrameId token found: {} of type: {}",self.cur_token.value, self.cur_token.token_type);
                stmt.original_obj = None;    
           },
        }

        StatementEnum::Update(stmt)
     }

     fn parse_select(self)-> StatementEnum{
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
           _=>{},
        } 

        StatementEnum::Select(stmt)
     }

     fn expect(&self, t: &str){
         if let Some(val) = self.keywords.get(&t){
             println!("Done expecting token: {}, found token type: {}",t, val);

             self.cur_token = self.peek_token;
             self.peek_token = self.lexer.next_token(&self.keywords);
         }else{
             eprintln!("Error occurred:  unexpected {} in the fql statement", t);
         }
     }

     fn match_next(&self, t: TokenType)->bool{
         if self.peek_token.token_type == t{
           return true;
         }
         eprintln!("Error occurred:  unexpected {} in the fql statement", t);
         
         false
     }
}


