use std::{collections::HashMap, fmt::Display, sync::LazyLock};

static KEYWORDS: LazyLock<HashMap<&'static str, TokenType>> = LazyLock::new(||{
   let mut keywords = HashMap::with_capacity(27_usize);
   
   keywords.insert("illegal", TokenType::Illegal);
   keywords.insert("string", TokenType::String);
   keywords.insert("star", TokenType::Star);
   keywords.insert("number", TokenType::Number);
   keywords.insert("semicolon", TokenType::Semicolon);
   keywords.insert("eq", TokenType::EQ);
   keywords.insert("eof", TokenType::EOF);
   keywords.insert("range", TokenType::Range);
   keywords.insert("objectimage", TokenType::ObjectImage);
   keywords.insert("frameid", TokenType::FrameId);
   keywords.insert("and", TokenType::And);
   keywords.insert("where", TokenType::Where);
   keywords.insert("classid", TokenType::ClassId);
   keywords.insert("classname", TokenType::ClassName);
   keywords.insert("video", TokenType::Video);
   keywords.insert("select", TokenType::Select);
   keywords.insert("or", TokenType::Or);
   keywords.insert("from", TokenType::From);
   keywords.insert("objectimages", TokenType::ObjectImages);
   keywords.insert("frames", TokenType::Frames);
   keywords.insert("detections", TokenType::Detections);
   keywords.insert("object", TokenType::Object);

   keywords
});

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug)]
pub struct Token{
    pub token_type: TokenType,
    pub value: String,
}

impl Display for Token{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}, {}", self.token_type, self.value)
    }
}

impl Token{
    pub fn empty()->Token{
        Token{
            token_type: TokenType::EOF,
            value: String::new(),
        }
    }
}

pub struct Lexer{
    input: String,
    left_pointer: u32,
    right_pointer: u32,
    pub ch: u8,
}


impl Lexer{
    pub fn empty()->Lexer{
        Lexer{
            input: String::new(),
            left_pointer: 0,
            right_pointer: 0,
            ch: 0,
        }
    }

    pub fn new(&mut self, input: String){
       self.input = input;  
       self.read_char();
        let ler = char::from(self.ch);
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

        let end = self.left_pointer as usize;
        let s = &self.input[start..end];
        s.to_string()
    }

    fn read_number(&mut self) ->String{
        let start = self.left_pointer as usize;
        while Lexer::is_digit(self.ch){
            self.read_char();
        }

        let end = self.left_pointer as usize;
        let s = &self.input[start..end];
        println!("read num: {}", s);
        s.to_string()
    }
   

   fn closing_quote(ch: u8)->bool{
      ch == b'\''
   }

   fn read_string(&mut self) -> String{
       self.read_char();

       let start = self.left_pointer as usize;
       while Lexer::is_letter(self.ch) || Lexer::is_digit(self.ch) || !Lexer::closing_quote(self.ch){
           self.read_char();
       }

       let end = self.left_pointer as usize;
       if Lexer::closing_quote(self.ch){
          //Current lexer char is a closing quote. CLOSING QUOTE FOUND, skipping.
            self.read_char(); 
       }

       let s = &self.input[start..end];
       s.to_string()
   }

   pub fn update_left_and_right(&mut self){
       self.read_char();
   }

   pub fn next_token(&mut self)->Token{
      let input_bytes = self.input.as_bytes();
      if self.ch == 0{
          if input_bytes[self.left_pointer as usize] == b';'{
             //End of statement found

              return Token { token_type: TokenType::Semicolon, value: ";".to_string() }
          } 
      }
            self.skip_whitespace();

      let ler = char::from(self.ch);
      //Letter representation(current lexer's char): {}", ler
      match self.ch{
          b';' =>Token { token_type:  TokenType::Semicolon, value: ";".to_string() },
          b'*' =>Token {token_type: TokenType::Star,
              value: "*".to_string()},
          b'=' => Token { token_type: TokenType::EQ, value: "=".to_string() },
          b'0' => Token { token_type: TokenType::EOF, value: "".to_string() },
          b'\'' =>{
               let tkn = Token { token_type: TokenType::String, value: self.read_string()};
              //AT STRING. found token: {}", tkn
                  tkn
          },
          _ =>{
              //println!("FOUND BLANK");
              if Self::is_letter(self.ch){
                 let letter = char::from(self.ch);
                // println!("left left_pointer at letter: {}", letter);
                 let value = self.read_identifier();
                 if let Some(token_type) = KEYWORDS.get(&value.as_str()){ 
                    return Token{token_type: token_type.clone(), value};
                 }else{
                     return  Token { token_type: TokenType::Illegal, value: String::from("")};
                 }
              }else if Self::is_digit(self.ch){
                  let num = self.read_number();
                  return Token{token_type: TokenType::Number, value: num};
              }

              //println!("must be illegal token..");
              Token{token_type: TokenType::Illegal, value: self.ch.to_string()}
             },
      }
   } 

  pub fn get_keyword(t: &str)->TokenType{
         if let Some(val) = KEYWORDS.get(&t){
             val.clone()        
         }else{
             TokenType::Illegal
        }
  }

}

