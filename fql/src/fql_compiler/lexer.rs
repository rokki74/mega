use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Debug)]
pub enum TokenType{
    Illegal,
    Number,
    Select,
    FrameId,
    ClassId,
    ClassName,
    From,
    Where,
    And,
    Or,
    ObjectImage,
    Object,
    Video,
    Range,
    Semicolon,
    String,
    Star,
    EQ,
    Comma,
    Frames,
    ObjectImages,
    Detections,
    EOF,
}

impl Display for TokenType{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}",self)
    }
}

#[derive(Clone)]
pub struct Token{
    pub token_type: TokenType,
    pub value: String,
}

impl Token{
    pub fn empty()->Token{
        Token{
            token_type: TokenType::EOF,
            value: String::new(),
        }
    }
}

pub struct Lexer<'a>{
    input: & 'a str,
    left_pointer: u32,
    right_pointer: u32,
    ch: u8,
}


impl <'a> Lexer<'a>{
    pub fn empty()->Lexer<'a>{
        Lexer{
            input: "",
            left_pointer: 0,
            right_pointer: 1,
            ch: 0,
        }
    }

    pub fn new(&mut self, input: &'a str){
       self.input = input;  
       self.read_char();
    }

    fn read_char(&mut self){
        let input_bytes = self.input.as_bytes();
        if self.right_pointer as usize >= input_bytes.len(){
            self.ch = 0;
        }else{
            self.ch = input_bytes[self.right_pointer as usize];
            self.left_pointer = self.right_pointer;
            self.right_pointer += 1;
        }
    }

    fn is_digit(ch: u8)->bool{
        ch >= b'0' && ch <= b'9'
    }

    fn is_letter(ch: u8)->bool{
       (ch >= b'a' && ch <= b'z') ||
       (ch >= b'A' && ch <= b'Z') || ch == b'_' 
    }

    fn skip_whitespace(&mut self){
       while self.ch == b' '|| self.ch == b'\t' || self.ch == b'\n' || self.ch == b'\r'{
           self.read_char();
       }
    }

    fn read_identifier(&mut self) -> String{
        let start = self.left_pointer as usize;
        while Lexer::is_letter(self.ch) || Lexer::is_digit(self.ch){
            self.read_char();
        }

        let end = self.right_pointer as usize;
        let s = &self.input[start..end];
        s.to_string()
    }

    fn read_number(&mut self) ->String{
        let start = self.left_pointer as usize;
        while Lexer::is_digit(self.ch){
            self.read_char();
        }

        let end = self.right_pointer as usize;
        let s = &self.input[start..end];
        s.to_string()
    }

   fn read_string(&mut self) -> String{
       self.read_char();

       let start = self.left_pointer as usize;
       while Lexer::is_letter(self.ch) || Lexer::is_digit(self.ch){
           self.read_char();
       }

       let end = self.right_pointer as usize;
       self.read_char();

       let s = &self.input[start..end];
       s.to_string()
   }

   pub fn next_token(&mut self, keywords: &HashMap<&str, TokenType>)->Token{
      self.skip_whitespace();

      match self.ch{
          b';' =>Token { token_type:  TokenType::Semicolon, value: ";".to_string() },
          b'*' =>Token {token_type: TokenType::Star,
              value: "*".to_string()},
          b'=' => Token { token_type: TokenType::EQ, value: "=".to_string() },
          b'0' => Token { token_type: TokenType::EOF, value: "".to_string() },
          b'\'' => Token { token_type: TokenType::String, value: self.read_string(), },
          _ =>{
              if Self::is_letter(self.ch){
                 let value = self.read_identifier();
                 if let Some(token_type) = keywords.get(&value){ 
                    return Token{token_type: token_type.clone(), value};
                 }else{
                     return  Token { token_type: TokenType::Illegal, value: String::from("")};
                 }
              }else if Self::is_digit(self.ch){
                  let num = self.read_number();
                  return Token{token_type: TokenType::Number, value: num};
              } return Token{token_type: TokenType::Illegal, value: self.ch.to_string()}; }, }; self.read_char(); return Token { token_type: TokenType::Illegal, value: "".to_string() } } 
}

