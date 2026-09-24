use std::{time::Duration};
use std::net::TcpStream;

use crate::{fql_compiler::{executor::{parse_media_timestamp}, lexer::{Lexer, Token, TokenType}}, video_src::ApiPref};

pub struct Parser{
    lexer: Lexer,
    peek_token: Token,
    cur_token: Token,
}

#[derive(Debug)]
pub enum Object {
    Frames,
    Image(String),
    Detections,
    ObjectImages,
}

pub enum FqlOutCome{
    NULL,
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Clone, Debug)]
pub enum BinaryOperation{
   Equal,
   And, 
   Or,
   Not,
}

impl BinaryOperation{
    pub fn precedence(b_op: BinaryOperation)->u32{
        match b_op{
            BinaryOperation::Or => 1,
            BinaryOperation::And => 2,
            BinaryOperation::Not => 3,
            BinaryOperation::Equal => 4,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expression{
    NumberLiteral(String),
    StringLiteral(String),
    //frameid, classname etc
    Identifier(String),
    Binary{
        left: Box<Expression>,
        op: BinaryOperation,
        right: Box<Expression>
    }  
}
                       
#[derive(Debug)]
pub struct SelectStatement{
    pub target: Object,
    pub url: Option<UrlSrc>,
    pub timeline: Option<(Duration, Duration)>,
    pub expr: Option<Expression>,
    pub preference: ApiPref,
}

impl SelectStatement{
    fn empty()->SelectStatement{
        SelectStatement { 
            target: Object::Frames,
            url: None,
            timeline: None,
            expr: None,
            preference: ApiPref::OCV,
        }
    }
}

impl Parser{
   pub fn empty()->Parser{
        Parser{
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

pub enum StatementEnum{
    SelectStatement(SelectStatement),
    VOID
}

impl Parser{
     pub fn parse<T: Fn(&mut TcpStream, &str)>(&mut self, query: String, stream: &mut TcpStream, messenger: T)->StatementEnum{
         self.lexer.new(query.clone());

         let token = self.lexer.next_token();
             
         self.cur_token = token.clone();
         self.peek_token = self.lexer.next_token();
         match self.cur_token.token_type{
             TokenType::Select => {
                 StatementEnum::SelectStatement(self.parse_select(stream, messenger))
             },
             _=>{ 
                 messenger(stream, "Expected an fql statement/query, cannot process a non-fql query input");
                 StatementEnum::VOID
             },
         }
    }

    fn parse_select<T: Fn(&mut TcpStream, &str)>(&mut self, stream: &mut TcpStream, messenger: T)-> SelectStatement{
        self.expect("select");
        let mut stmt = SelectStatement::empty();
        match self.cur_token.token_type{
           TokenType::Star =>{
               self.expect("star");
               
               match self.cur_token.token_type{
                   TokenType::Frames =>{
                      stmt.target = Object::Frames
                   },
                   TokenType::ObjectImages =>{
                       stmt.target = Object::ObjectImages
                   },
                   _=>{
                       let s = format!("Error parsing statement, unexpected token,  Instead found the token: {} expected either Frames or object images", self.cur_token);
                       messenger(stream, &s);
                   }
               }

               self.expect("from");

               /*
               if !self.match_peek(TokenType::Video){
                   panic!("Incorrect syntax, needed a video instead.");
               }
               */
               self.expect("video");
               
               self.lexer.next_token();

               self.expect("eq");

               
               let url = self.cur_token.value.clone();
               stmt.url = Some(UrlSrc::Vid(url.clone()));
               self.expect("string");

               while !self.match_peek(TokenType::Semicolon){
                  match self.cur_token.token_type{
                    TokenType::Range =>{
                       self.expect("range");

                       let start = parse_media_timestamp(&self.cur_token.value);

                       self.expect("string");
                       let end = parse_media_timestamp(&self.cur_token.value);
                       self.expect("string");

                       stmt.timeline = Some((start, end));
                    },
                    TokenType::Where =>{
                      self.expect("where");

                      stmt.expr = Some(self.parse_expr(stream, &messenger));
                    },
                    _=>{let err = format!("Expected either a where clause or range statement!, found {} of type {}", self.cur_token.value, self.cur_token.token_type);
                        messenger(stream, &err);
                    },
                  }
               }
           },
           //Make detections
           TokenType::Detections =>{
               self.expect("detections");
               self.expect("from");

               println!("CURRENT STATE: cur: {}, peek: {}", self.cur_token, self.peek_token);

               match self.cur_token.token_type{
                   TokenType::Object=>{
                       self.expect("object");
                       self.expect("eq");
                         let url = self.cur_token.value.clone();
                         println!("image's url: {}", url);
                           stmt.url = Some(UrlSrc::Img(url));
                   },
                   TokenType::Video =>{
                         self.expect("video");
                            let url = self.cur_token.value.clone();
                         println!("video's url: {}", url);
                           stmt.url = Some(UrlSrc::Vid(url));
                   },
                   _=>{
                       messenger(stream, "Invalid token found detections can only work if in Object(images including jpg etc) or Video(e.g .mp4 files) types");
                       panic!("Error parsing a detection object");
                   }
               }

               stmt.target = Object::Detections;
           },
           _=>{println!("THE escaping part......")},
        } 

        stmt
     }

     fn parse_expr<T: Fn(&mut TcpStream, &str)>(&mut self, stream: &mut TcpStream, messenger: T)-> Expression{
         let ident = self.lexer.next_token().value;
         let left = Expression::Identifier(ident);

         let tok = self.lexer.next_token();
         match tok.token_type{
             TokenType::EQ =>{
                 let op = BinaryOperation::Equal;
                 let right = self.parse_expr(stream, messenger);

                 Expression::Binary { left: Box::new(left), op, right: Box::new(right) }
             },
            TokenType::String =>{
                 let expr = Expression::StringLiteral(self.cur_token.value.clone());
                 expr
            },
            TokenType::Number =>{
                 let expr = Expression::NumberLiteral(self.cur_token.value.clone());
                 expr
            },
            TokenType::Or =>{
                let op = BinaryOperation::Or;
                let right = self.parse_expr(stream, messenger);
                
                Expression::Binary { left: Box::new(left), op, right: Box::new(right)}
            },
            TokenType::And =>{
                let op = BinaryOperation::And;
                let right = self.parse_expr(stream, messenger);

                Expression::Binary { left: Box::new(left), op, right: Box::new(right) }
            }
            _=>{
                let err = format!("Expected either a number or string, found {}", self.cur_token.value);
                messenger(stream, &err);
                panic!("Unable to process the expression");
            }
         }
     }

     fn expect(&mut self, t: &str){
         let tok = Lexer::get_keyword(t);
         if tok == self.cur_token.token_type{
             println!("Expected token doesn't match what's contained in current token i.e cur_tok: ({}), keywrd: ({})", self.cur_token, tok);
         }

         match tok{
             TokenType::Illegal =>{
             panic!("Error occurred:  unexpected {} in the fql statement", t);
             },
             _ => {
             self.cur_token = self.peek_token.clone();
             //caller to next_token function must be responsible for calling the read_char Just like new lexer did so as to
             //make it work for everyone elese thus after creating a new lexer next reads need to
             //utilise this expect func as it is the ony interface to facilitate before reading
             //next_token
             self.lexer.update_left_and_right();
             self.peek_token = self.lexer.next_token();
             }, 
         }
     }

     fn match_peek(&self, t: TokenType)->bool{
        t == self.peek_token.token_type
     }
}

