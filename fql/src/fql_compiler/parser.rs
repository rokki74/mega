use std::collections::HashMap;
use crate::fql_compiler::lexer::{TokenType, Lexer};

struct Parser{
    keywords: &HashMap<&str, TokenType>,
}

impl Parser{
     fn new()->Parser{  
       let mut keywords: HashMap<&str, TokenType> = HashMap::with_capacity(27_usize);
       
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

       Parser { keywords: &keywords}
     }


     pub fn parse(self, fql: String){
         let lexer = Lexer::new(&fql);
         loop{
             let token = lexer.next_token(self.keywords);

             if token.TokenType != TokenType::Illegal{
                ;write!()
             }else{
                 return;
             }
         }
     }
}


