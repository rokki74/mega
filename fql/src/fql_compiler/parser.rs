use std::{collections::HashMap, time::Duration};
use crate::{fql_compiler::{executor::{parse_media_timestamp}, lexer::{Lexer, Token, TokenType}}, video_src::ApiPref};

pub struct Parser{
    lexer: Lexer,
    peek_token: Token,
    cur_token: Token,
}

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

#[derive(Clone)]
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

#[derive(Clone)]
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

type FrameId = usize;
type URL = String;
type VidUrl = String;
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

pub enum StatementEnum{
    SelectStatement(SelectStatement)
}

impl Parser{
     pub fn parse(&mut self, query: String)->StatementEnum{
         self.lexer.new(query.clone());
         let token = self.lexer.next_token();

         println!("handling the token: token val: {}, token_type: {} Inside the parser", token.value, token.token_type);
             
         match token.token_type{
             TokenType::Select => {
                 StatementEnum::SelectStatement(self.parse_select())
             },
             _=>{ 
                 println!("Illegal value {} in statement: {}", token.value, query);
                 panic!("Expected an fql statement/query, cannot process a non-fql query input");
             },
         }
    }

    fn parse_select(&mut self)-> SelectStatement{
        self.expect("select");
       
        let mut stmt = SelectStatement::empty();
        match self.cur_token.token_type{
           TokenType::Star =>{
               self.expect("*");
               
               match self.cur_token.token_type{
                   TokenType::Frames =>{
                      stmt.target = Object::Frames
                   },
                   TokenType::ObjectImages =>{
                       stmt.target = Object::ObjectImages
                   },
                   _=>{
                       panic!("Error parsing statement, unexpected token")
                   }
               }

               self.expect("from");

               if !self.match_peek(TokenType::Video){
                   panic!("Incorrect syntax, needed a video instead.");
               }
               
               self.lexer.next_token();
               if !self.match_peek(TokenType::EQ){
                   panic!("Incorrect syntax, needed an equal operation/sign");
               }

               self.lexer.next_token();
               stmt.url = Some(UrlSrc::Vid(self.cur_token.value.clone()));

               while !self.match_peek(TokenType::Semicolon){
                  self.lexer.next_token();
                  
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

                      stmt.expr = Some(self.parse_expr());
                    },
                    _=>{panic!("Expected either a where clause or range statement!, found {} of type {}", self.cur_token.value, self.cur_token.token_type);},
                  }
               }
           },
           //Make detections on a single image.
           TokenType::Detections =>{
               self.lexer.next_token();
               self.expect("from");

               if !self.match_peek(TokenType::Object){
                   panic!("Expected Object");
               }
               self.lexer.next_token();

               self.expect("=");
               let url = self.cur_token.value.clone();
               stmt.url = Some(UrlSrc::Vid(url));
               stmt.target = Object::Detections;
           },
           _=>{},
        } 

        stmt
     }

     fn parse_expr(&mut self)-> Expression{
         let ident = self.lexer.next_token().value;
         let left = Expression::Identifier(ident);

         let tok = self.lexer.next_token();
         match tok.token_type{
             TokenType::EQ =>{
                 let op = BinaryOperation::Equal;
                 let right = self.parse_expr();

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
                let right = self.parse_expr();
                
                Expression::Binary { left: Box::new(left), op, right: Box::new(right)}
            },
            TokenType::And =>{
                let op = BinaryOperation::And;
                let right = self.parse_expr();

                Expression::Binary { left: Box::new(left), op, right: Box::new(right) }
            }
            _=>panic!("Expected either a number or string, found {}", self.cur_token.value),
         }
     }

     fn expect(&mut self, t: &str){
         let tok = Lexer::get_keyword(t);
         match tok{
             TokenType::Illegal =>{
             panic!("Error occurred:  unexpected {} in the fql statement", t);
             },
             _ => {
             self.cur_token = self.peek_token.clone();
             self.peek_token = self.lexer.next_token();
             }, 
         }
     }

     fn match_peek(&self, t: TokenType)->bool{
         const VAL: TokenType = self.peek_token.token_type.clone();
         match t{
           VAL => {println!("matched a peek token"); true},
           _ => {
               eprintln!("Error occurred:  unexpected {} in the fql statement", t);
         
               false
            },
         }
     }
}

