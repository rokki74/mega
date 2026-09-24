use ort::session;
use std::{path::Path};
use crate::{detection::Detection, fql_compiler::{lexer::{TokenType, Lexer}, parser::{BinaryOperation, Expression, FqlOutCome, Object, Parser, SelectStatement, StatementEnum, UrlSrc}}, image_src, model_session};
use std::{io::{self, Write}, net::TcpStream, time::Duration};
use crate::{video_src::{VideoSrc, FrameIter}, frame::{Frame}};

pub struct Executor<'a>{
  parser: Parser,
  writer: &'a mut TcpStream,
}

#[derive(Debug)]
enum FqlOutputType {
    SNAPSHOTS,
    DETECTIONS,
    FRAMES,
}

#[derive(Debug)]
pub struct FqlOutput{
    fql_out_type: FqlOutputType,
    range: Option<(Duration, Duration)>,
    pub finds: Option<Expression>,
}

pub enum EvaluationValue{
    Boolean(bool),
    Integer(u64),
    String(String),
    Null,
}

fn split_fql_statement(input: &String)->Vec<String>{
    input.split_inclusive(";").map(|sp| sp.to_string()).collect()
}

impl <'a> Executor<'a>{
    pub fn new(writer: &'a mut TcpStream)->Executor<'a>{
        let _ = writer.write_all(b"__Welcome to FQL__");
        Self{
         parser: Parser::empty(),
         writer,
       }
    }

    pub fn send_message(writer: &mut TcpStream, msg: &str){
        let msg = msg.to_string() + "\n";
        let _ = writer.write_all(msg.as_bytes());
    }

    pub fn execute(&mut self, fql: &String){
       let queries = split_fql_statement(fql);

       let s_stream = self.writer.try_clone();
       let mut writer = match s_stream{
           Ok(s) => s,
           Err(e) => panic!("Unable to clone tcp stream, due to err: {}", e),
       };

       for query in queries{
           //println!("AT EXECUTOR, PRCESSING QUERY: {}", query);
           let stmt = self.parser.parse(query.to_string(),&mut writer, Self::send_message);
           let fql_outcm = stmt.execute();

           println!("LEN OF OUTCOMES AT THE END IS: {}", fql_outcm.len());
           for outcm in fql_outcm{
              self.send_back_result(outcm);
           }
       }
    }

    pub fn evaluate(expr: &Expression, detection: &Detection)->EvaluationValue{
           SelectStatement::evaluate(expr.clone(), detection)
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
            },
            FqlOutCome::NULL =>{
                let _ = self.writer.write_all(b"No results, execution complete");
                 let _ = self.writer.flush();
            }
        }
    }
}

pub trait Statement{
    fn execute(&self)->Vec<FqlOutCome>{
       //println!("executing statement");
       vec![FqlOutCome::NULL]
    }
}

impl Statement for StatementEnum{
    fn execute(&self)->Vec<FqlOutCome>{
        match self{
            StatementEnum::SelectStatement(stmt) => {
                let _ = io::stdout().flush();
                stmt.execute()
            },
            StatementEnum::VOID =>{
                println!("(Executor/Statement) statement enum is void..");
                let _ = io::stdout().flush();
                vec![FqlOutCome::NULL]
            },
        }
    }
}

impl Statement for SelectStatement{
    fn execute(&self) -> Vec<FqlOutCome>{
        if let Some(url_src) = &self.url{
            let base_url = "/home/nines/Desktop/rusty/mega/fql/assets/";
            let _ = io::stdout().flush();
            match url_src{
                UrlSrc::Vid(file_src) =>{
                    println!("The url_src is vid");
                    let file_src = &(base_url.to_string() + file_src);
                    let file_path = Path::new(&file_src);
                      if file_path.exists(){
                         let (frms_iter, mut session) = VideoSrc::open(&file_src, &self.preference); 
                         
                        let _ = io::stdout().flush();
                        self.execute_frames(frms_iter, &mut session)

                      }else{
                          //println!("Video file provided: {} doesn't exist", file_src);
                          vec![FqlOutCome::NULL]
                      }
                },
                UrlSrc::Img(img_url) =>{
                    println!("The url_src is img");
                   let img_url = &(base_url.to_string() + img_url);
                   let (frm, mut session) =  image_src::open_image(&img_url);
                    let _ = io::stdout().flush();
                   if let Some(f) = self.execute_frame(frm, &mut session){
                      vec![f]
                   }else{
                       vec![FqlOutCome::NULL]
                   }
                },
            }
        }else{
           vec![FqlOutCome::NULL]
        }
    }
}

impl SelectStatement{
    fn execute_frame(&self, frame: Frame, session: &mut session::Session)->Option<FqlOutCome>{
        let fql_out_type = match self.target{
            Object::ObjectImages => FqlOutputType::SNAPSHOTS,
            Object::Detections => FqlOutputType::DETECTIONS,
            _=> FqlOutputType::FRAMES,
        };
        

        let range = self.timeline;
        let finds = self.expr.clone();
        
        let fql_out = FqlOutput{
            fql_out_type,
            range,
            finds,
        };

        println!("(FQLOUTPUT)  Processing your wants, fql_out: {:#?}", fql_out);

        let _ = io::stdout().flush();
        Some(frame.process_frame(session, fql_out))
    }

    fn execute_frames(&self, frms_iter: FrameIter, session: &mut session::Session)->Vec<FqlOutCome>{
       let mut out: Vec<FqlOutCome> = Vec::new();
       let max_counts = 50;
        let mut counts = 0;
       match frms_iter{
            FrameIter::Ocv(ocvs) =>{
                //returning at 50
                for frm in ocvs{
                    counts +=1;
                    if counts > max_counts{
                        println!("{} reached..", max_counts);
                        break;
                    }
                    
                    if let Some(f_out) = self.execute_frame(frm, session){
                        out.push(f_out);
                    }else{
                        out.push(FqlOutCome::NULL);
                    }
                    let _ = io::stdout().flush();
                }

                out
            },
            FrameIter::Ffm(ffms)=>{
                for frm in ffms{
                    //returning at 50
                    counts +=1;
                    if counts > max_counts{
                        return out;
                    }
                    if let Some(f_out) = self.execute_frame(frm, session){
                        out.push(f_out);
                    }else{
                        println!("pushing null into outcms");
                        out.push(FqlOutCome::NULL);
                    }
                    let _ = io::stdout().flush();
                }

                out
            },
       }
    }

    pub fn evaluate(expr: Expression, detection: &Detection)->EvaluationValue{
       let coco = Detection::fill_coco_classes_map();
       match expr{
           Expression::Binary { left, op, right } =>{
              let left_res = Self::evaluate(*left, detection);
              let right_res = Self::evaluate(*right, detection);

              match op{
                  BinaryOperation::And =>{
                      match(left_res, right_res){
                          (EvaluationValue::Boolean(a), EvaluationValue::Boolean(b)) =>{
                              EvaluationValue::Boolean(a & b)
                          },
                          _=>EvaluationValue::Null,
                      }
                  },
                  BinaryOperation::Not =>{ 
                      match right_res{
                          EvaluationValue::Boolean(a) =>{
                            EvaluationValue::Boolean(!a)
                          },
                          _=>EvaluationValue::Null,
                      }
                  },
                  BinaryOperation::Equal =>{
                      match(left_res, right_res){
                          (EvaluationValue::Boolean(a), EvaluationValue::Boolean(b))=>{
                              EvaluationValue::Boolean(a == b)
                          },
                          (EvaluationValue::String(a), EvaluationValue::String(b))=>{
                              let k = Lexer::get_keyword(&a);

                              match k{
                                  TokenType::Range =>{
                                     let (start, end) = Self::parse_timeline_string(&b);
                                     if let Some(t) = detection.timestamp{
                                        if t >= start && t <= end{
                                            EvaluationValue::Boolean(true)
                                        }else{
                                            EvaluationValue::Boolean(false)
                                        }
                                     }else{
                                         EvaluationValue::Null
                                     }
                                  },
                                  TokenType::Object =>{
                                     let (img_ob, _) = image_src::open_image(&b); 
                                     let match_percentage = Self::cmp_frame_and_detection(&img_ob, &detection);
                                     if match_percentage > 0.5{
                                         EvaluationValue::Boolean(true)
                                     }else{
                                         EvaluationValue::Boolean(false)
                                     }
                                  },
                                  TokenType::ClassName =>{
                                     let classid = if let Some(id) = coco.get(&b){
                                                      id
                                                   }else{
                                                      panic!("No such classname for fql detections");
                                                   };

                                     if detection.class_id == *classid{
                                         EvaluationValue::Boolean(true)
                                     }else{
                                         EvaluationValue::Boolean(false)
                                     }
                                  },
                                  _=>EvaluationValue::Null
                              }
                          },
                          (EvaluationValue::Integer(a), EvaluationValue::Integer(b))=>{
                              EvaluationValue::Boolean(a == b)
                          },
                          _=>EvaluationValue::Null
                      }
                  },
                  BinaryOperation::Or =>{
                      match (left_res, right_res){
                          (EvaluationValue::Boolean(a), EvaluationValue::Boolean(b))=>{
                              EvaluationValue::Boolean(a || b)
                          },
                          _=>{
                              EvaluationValue::Null
                          }
                      }
                  },
              }
           },
           Expression::Identifier(ident) =>{
              EvaluationValue::String(ident)
           },
           Expression::NumberLiteral(num)=>{
              let n: u64 = num.parse().expect("Unable to convert string into Integer i.e u64 from expressions number literal"); 
              EvaluationValue::Integer(n)
           },
           Expression::StringLiteral(val)=>{
              EvaluationValue::String(val)
           },
       }
    }

    pub fn cmp_frame_and_detection(frame: &Frame, det: &Detection)->f32{
       let mut session = model_session::init_yolo_sesion(true);
       let dets = frame.process_frame_raw(&mut session);
       let mut threshold_dets =0;
       let total_dets = dets.len();
       for det2 in dets{
         if Detection::match_detections(det, &det2){
            threshold_dets += 1;
         }
       }
       
       ((threshold_dets/total_dets) * 100) as f32
    }

    fn parse_timeline_string(timeline: &String)->(Duration, Duration){
       let mut parts = timeline.split_whitespace();
       
       let start = parts.next().expect("Missing start time!");
       let end = parts.next().expect("Missing end time!");

       let start_dur = parse_media_timestamp(start);
       let end_dur = parse_media_timestamp(end);

       (start_dur, end_dur)
    }
}

pub fn parse_media_timestamp(timestamp: &str)->Duration{
        let parts: Vec<&str> = timestamp.split(':').collect();

        let (hours, minutes, seconds) = match parts.len(){
            2  =>{
                 let mins: u64 = parts[0].parse().expect("Invalid Minutes");
                 let secs: u64 = parts[1].parse().expect("Invalid Seconds");

                 (0, mins, secs)
            },
            3 =>{
                let hrs: u64 = parts[0].parse().expect("Invalid Hours");
                let mins: u64 = parts[1].parse().expect("Invalid Minutes");
                let secs: u64 = parts[3].parse().expect("Invalid Seconds");

                (hrs, mins, secs)
            },
            _=>{
                panic!("Invalid value for range query, could not parse the range timestamp!");
            },
        };

        let dur_secs = (hours *3600) + (minutes *60) + seconds;
        Duration::from_secs(dur_secs)
    }
