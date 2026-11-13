use ort::session;
use std::{path::Path};
use crate::{detection::Detection, fql_compiler::{lexer::{TokenType, Lexer}, parser::{BinaryOperation, Expression, FqlOutCome, Object, Parser, SelectStatement, StatementEnum, UrlSrc}}, image_src, model_session};
use std::{io::{self, Write}, net::TcpStream, time::Duration};
use crate::{video_src::{VideoSrc, FrameIter}, frame::{Frame}};
use std::fs::{OpenOptions};
use crate::{detection::{detect, OutCome}};
use crate::fql_session::{FqlSessionUsability, FqlSession};

const BASE_DIR: &str = "/home/nines/Desktop/rusty/fql_data";

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

const MAX_OUTCOME_RESULTS:usize = 100;

pub struct FqlOutcomeBuf<'a>{
   pub items: Vec<String>,
   fql_session: &'a mut FqlSession,
   stream: &'a mut TcpStream,
}

impl <'a>FqlOutcomeBuf<'a>{
    pub fn new(stream: &'a mut TcpStream, fql_session: &'a mut FqlSession)->Self{
        FqlOutcomeBuf {
            items: Vec::new(),
            stream,
            fql_session
        }
    }


    pub fn add_detection(&mut self, det: &Detection){
        let outcome_det = detect(&det);
        if let OutCome::Res(fmted_det) = outcome_det{
           if self.items.len() >= MAX_OUTCOME_RESULTS{
               self.send_message(&fmted_det);

              self.items.clear();

              //sends both analysis and detections vecs
              self.fql_session.save_session();
              self.fql_session.clear_data();
           }

           self.items.push(fmted_det);
        }else{
            self.send_message("No results available for frame!");
        }
    }

    #[inline]
    pub fn send_message(&mut self, message: &str){
        self.stream.write_all(message.as_bytes());
    }
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
           let stmt = self.parser.parse(query.to_string(),&mut writer, Self::send_message);
           let fql_outcm = stmt.execute(&mut writer);

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
            },
            FqlOutCome::CLEARED =>{
                let _ = self.writer.write_all(b"Results sent!, cleared from buffer");
                 let _ = self.writer.flush();
            }
        }
    }
}

pub trait Statement{
    fn execute(&self, _writer: &mut TcpStream)->Vec<FqlOutCome>{
       //println!("executing statement");
       vec![FqlOutCome::NULL]
    }
}

impl Statement for StatementEnum{
    fn execute(&self, writer: &mut TcpStream)->Vec<FqlOutCome>{
        match self{
            StatementEnum::SelectStatement(stmt) => {
                let _ = io::stdout().flush();
                stmt.execute(writer)
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
    fn execute(&self, writer: &mut TcpStream) -> Vec<FqlOutCome>{
        if let Some(url_src) = &self.url{
            let base_url = "/home/nines/Desktop/rusty/mega/fql/assets/";
            let _ = io::stdout().flush();

            match url_src{
                UrlSrc::Vid(file_src) =>{
                    println!("The url_src is vid");
                    let file_src = &(base_url.to_string() + file_src);
                    let (mut fql_sess,usability) = FqlSession::get_session(file_src);
                    let mut sess_buf = FqlOutcomeBuf::new(writer, &mut fql_sess);
                    //FIX ME: consume usability
                    //AFTERARDS EITHER CHOOSE TO USE PRE SAVED SESSION DATA i.e detections, frames,
                    //analysis etc OR NOT. CURRENTLY NOT
                   
                   let mut session = model_session::init_yolo_sesion(true);
                    match usability{
                       FqlSessionUsability::ANALYSISUSABLE =>{
                           //So already there were analysis and now if the current request is same
                           //then return it..
                           if self.old_analysis{
                              let iter = sess_buf.fql_session.read_analysis().unwrap(); 
                              for res in iter{
                                  let analysis = res.unwrap();
                                  sess_buf.send_message(&analysis);
                              }
                           }
                       },
                       FqlSessionUsability::DETECTIONUSABLE =>{
                            //So already there were detections and now if the current request is same
                           //then return it..
                           match self.target{
                               Object::Detections =>{
                                   let iter = sess_buf.fql_session.read_detections().unwrap(); 
                                   if let Some(tm) = self.timeline{
                                      let (start, end) = tm;
                                       //read out the read_start, end_start from detections file
                                         for res in iter{
                                             let det = res.unwrap();
                                             if let Some(timestamp) = det.timestamp{
                                                 //FIX: need better analytics of getting the exact
                                                 //needed detections
                                                 if start <= timestamp && end >= timestamp{
                                                    let outcm = detect(&det);
                                                     match outcm{
                                                         OutCome::Res(out)=>{
                                                              sess_buf.send_message(&out);
                                                         },
                                                         OutCome::NULL=>{sess_buf.send_message("Found null outcome from detection, skipping read det");},
                                                     }
                                                 }
                                              }
                                         } 
                                   }else{
                                       for res in iter{
                                             let det = res.unwrap();
                                             let outcm = detect(&det);
                                             match outcm{
                                                 OutCome::Res(out)=>{
                                                     sess_buf.send_message(&out);
                                                 },
                                                 OutCome::NULL=>{println!("skipping read det as is outcome null");},
                                             }
                                       }
                                   }

                               },
                               _=>(),
                            }
                       }, 
                       FqlSessionUsability::FRAMEUSABLE =>{
                           //Implicitly using frms_iter from file, executing them ..
                            let frms_iter = sess_buf.fql_session.read_frames(); 
                            let outcome = self.execute_frames(frms_iter, &mut session, &mut sess_buf);
                            return vec![outcome]
                       },
                       _ =>(),
                   }

                    let file_path = Path::new(&file_src);
                      if file_path.exists(){
                         let (frms_iter, _) = VideoSrc::open(&file_src, &self.preference); 
                         
                        let outcome = self.execute_frames(frms_iter, &mut session, &mut sess_buf);
                        vec![outcome]
                      }else{

                          //println!("Video file provided: {} doesn't exist", file_src);
                          vec![FqlOutCome::NULL]
                      }
                },
                UrlSrc::Img(img_url) =>{
                    println!("The url_src is img");
                    let img_url = &(base_url.to_string() + img_url);
                    let mut fql_sess = FqlSession::get_empty();
                    let mut sess_buf = FqlOutcomeBuf::new(writer, &mut fql_sess);
                    let (frm, mut session) =  image_src::open_image(&img_url);
                    let _ = io::stdout().flush();
                    let outcome = self.execute_frame(frm, &mut session, &mut sess_buf);
                    vec![outcome]
                },
                UrlSrc::Cam(cam_id)=>{
                    //Consume from camera! finally coded it, yes
                    let (frms_iter, mut session) = VideoSrc::open(&cam_id, &self.preference); 
                    let mut fql_sess = FqlSession::get_empty();
                    let mut sess_buf = FqlOutcomeBuf::new(writer, &mut fql_sess);
                    let outcome = self.execute_frames(frms_iter, &mut session, &mut sess_buf);
                    vec![outcome]
                },
            }
        }else{
           vec![FqlOutCome::NULL]
        }
    }
}

impl SelectStatement{
    fn execute_frame(&self, frame: Frame, session: &mut session::Session, sess_buf: &mut FqlOutcomeBuf)->FqlOutCome{
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
        frame.process_frame(session, fql_out, sess_buf);

        FqlOutCome::Multiple(sess_buf.items.clone())
    }

    fn execute_frames(&self, frms_iter: FrameIter, session: &mut session::Session, sess_buf: &mut FqlOutcomeBuf)->FqlOutCome{
       let frames_file = BASE_DIR.to_string() + &sess_buf.fql_session.name + ".frames";
       let mut file = OpenOptions::new()
           .create(true)
           .write(true)
           .append(true)
           .open(frames_file).unwrap();
       

       match frms_iter{
            FrameIter::Ocv(ocvs) =>{
                for frm in ocvs{
                    sess_buf.fql_session.save_frame(&mut file, &frm);
                    let _ = self.execute_frame(frm, session, sess_buf);
                }
            },
            FrameIter::Ffm(ffms)=>{
                for frm in ffms{
                    sess_buf.fql_session.save_frame(&mut file, &frm);
                    let _ = self.execute_frame(frm, session, sess_buf);
                }
            },
            FrameIter::FileSourced(iter)=>{
                for frm in iter{
                    let _ = self.execute_frame(frm, session, sess_buf);
                }
            },
            FrameIter::Cam(cam_iter)=>{
                for frm in cam_iter{
                    sess_buf.fql_session.save_frame(&mut file, &frm);
                    let _ = self.execute_frame(frm, session, sess_buf);
                }
            }
       }

       FqlOutCome::CLEARED
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

pub fn parse_media_timestamps(start: &str, end: &str)->(Duration, Duration){
    (parse_media_timestamp(start), parse_media_timestamp(end))
}


pub fn parse_media_timestamp(timestamp: &str)->Duration{
        let parts: Vec<&str> = timestamp.split(':').collect();

        let (hours, minutes, seconds) = match parts.len(){
            1 =>{
                let secs: u64 = parts[0].parse().expect("Invalid Seconds");
                (0, 0, secs)
            }
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
