use ort::session;
use crate::{detection::Detection, fql_compiler::{lexer::TokenType, parser::{BinaryOperation, Expression, FqlOutCome, Object, Parser, SelectStatement, StatementEnum, UrlSrc}}, image_src, model_session};
use std::{io::Write, net::TcpStream, time::Duration};
use crate::{video_src::{VideoSrc, FrameIter}, frame::{Frame}};

pub struct Executor<'a>{
  parser: Parser<'a>,
  writer: &'a mut TcpStream,
}

enum FqlOutputType {
    SNAPSHOTS,
    DETECTIONS,
    FRAMES,
}

pub struct FqlOutput{
    fql_out_type: FqlOutputType,
    range: (Duration, Duration),
    pub finds: Expression,
}

pub enum EvaluationValue{
    Boolean(bool),
    Integer(u64),
    String(String),
    Null,
}

impl <'a> Executor<'a>{
    pub fn new(writer: &'a mut TcpStream)->Executor<'a>{
        let _ = writer.write_all(b"__Welcome to FQL__");
        Self{
         parser: Parser::empty(),
         writer,
       }
    }

    pub fn execute(&mut self, fql: String){
       let queries = fql.split(";");

       for query in queries{
           let stmt = self.parser.parse(query.to_string());
           let fql_outcm = stmt.execute();

           if let Some(outcome) = fql_outcm{
             self.send_back_result(outcome);
           }else{
               self.writer.write_all(b"Displaying 0 results");
               self.writer.flush();
           }
       }
    }

    pub fn evaluate(expr: Expression, detection: &Detection)->EvaluationValue{
           SelectStatement::evaluate(expr, detection)
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
       println!("executing statement");
       vec![FqlOutCome::NULL]
    }
}

impl Statement for StatementEnum{
    fn execute(&self)->Vec<FqlOutCome>{
        match self{
            StatementEnum::SelectStatement(stmt) => stmt.execute(),
        }
    }
}

impl Statement for SelectStatement{
    fn execute(&self) -> Vec<FqlOutCome>{
        if let Some(url_src) = &self.url{
            match url_src{
                UrlSrc::Vid(vid_url) =>{
                   let (frms_iter, mut session) = VideoSrc::open(&vid_url, &self.preference); 

                   self.execute_frames(frms_iter, &mut session)
                },
                UrlSrc::Img(img_url) =>{
                   let (frm, mut session) =  image_src::open_image(&img_url);
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

        let range = self.timeline?;
        let finds = self.expr.clone()?;
        
        let fql_out = FqlOutput{
            fql_out_type,
            range,
            finds,
        };

        let results = frame.process_frame(session, true, fql_out);
        Some(results)
    }

    fn execute_frames(&self, frms_iter: FrameIter, session: &mut session::Session)->Vec<FqlOutCome>{
       let mut out: Vec<FqlOutCome> = Vec::new();
       match frms_iter{
            FrameIter::Ocv(ocvs) =>{
                for frm in ocvs{
                    if let Some(f_out) = self.execute_frame(frm, session){
                        out.push(f_out);
                    }else{
                        out.push(FqlOutCome::NULL);
                    }
                }

                out
            },
            FrameIter::Ffm(ffms)=>{
                for frm in ffms{
                    if let Some(f_out) = self.execute_frame(frm, session){
                        out.push(f_out);
                    }else{
                        out.push(FqlOutCome::NULL);
                    }
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
                              let parser = Parser::empty();
                              let k = parser.get_keyword(&a);

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
       let parts = timeline.split_whitespace();
       
       let start = parts[0];
       let end = parts[1];

       (Duration::from(start), Duration::from(end))
    }
}
