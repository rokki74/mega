use std::io::{self, BufReader, BufRead, Read, Write};
use std::fs::{self, File, OpenOptions};
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;
use crate::detection::Detection;
use crate::frame::Frame;
use crate::video_src::FrameIter;

pub const BASE_DIR: &str = "/home/nines/Desktop/rusty/fql_data";

struct FqlSessionMetadata{
   cam_sourced: bool,
   user_generated: bool, //did the user explicitly request at parser or the system needed to build
                         //for reliefing the memory i.e of too big vec<frames> in-mem etc
   analyzed: bool, //directory i.e analysis directory containing varying amnts of analyses
   detected: bool, //file i.e detections.txt
   framed: bool, //file
   completed: bool //Did processing in either stage finish i.e completely framed thus skip on next
                   //processing
    //A 7th and 8th bit-flag later if need be
}

pub enum FqlSessionUsability{
    FRAMEUSABLE,
    DETECTIONUSABLE,
    ANALYSISUSABLE,
    NONEUSABLE,
}

struct FqlSessionOutput{
    analysis_out: Option<Vec<String>>,
    detections_out: Option<Vec<Detection>>,
}

impl FqlSessionOutput{
    fn empty()->FqlSessionOutput{
       FqlSessionOutput { analysis_out: None, detections_out: None}
    }
}

pub struct FqlSession{
    pub name: String,//Also the directory name holding up the files i.e analysis, frames and detections
    //for this session 
    meta: FqlSessionMetadata,
    out: FqlSessionOutput
}

pub fn create_session(name: &String, cam_sourced: bool, user_generated: bool)->FqlSession{
    let meta = FqlSessionMetadata{
           cam_sourced,
           user_generated,
           analyzed: false,
           detected: false,
           framed: false,
           completed: false,
    };

    let mut sess = FqlSession{
        name: name.clone(),
        meta,
        out: FqlSessionOutput::empty()
    };
    
    let sess_dir = BASE_DIR.to_string() + &name;
    fs::create_dir_all(sess_dir);

    sess.save_session();
    sess
}

//FIX ME: bit flags
pub fn open_session(sess_file: &String, dir_name: String)->FqlSession{
       let mut file = File::open(sess_file).unwrap();
       let mut buf = [0u8, 1];
       file.read_exact(&mut buf).expect("unable to read back metadata(byte) from the session file");

       let metadata = buf[0];

       let meta = FqlSessionMetadata{
           cam_sourced: (metadata & (1u8 <<7)) != 0, 
           user_generated: (metadata & (1u8 << 6)) !=0, 
           analyzed: (metadata & (1u8 << 5)) != 0, 
           detected: (metadata & (1u8 << 4)) != 0, 
           framed: (metadata & (1u8 << 3)) != 0, 
           completed: (metadata & (1u8 << 2)) != 0,
       };

       FqlSession{
           name: dir_name,
           meta,
           out: FqlSessionOutput { analysis_out: None, detections_out: None }
       } 
}

//FIX ME: implement
pub fn delete_session(name: String){
  let sess_dir = BASE_DIR.to_string() + &name;
  fs::remove_dir_all(sess_dir);
  println!("session deleted!");
}

impl FqlSession{
    pub fn get_empty()->FqlSession{
        FqlSession { 
          name: "empty".to_string(),
          meta: FqlSessionMetadata { cam_sourced: false, user_generated: false, analyzed: false, detected: false,
                                  framed: false, completed: false 
              },
          out: FqlSessionOutput { analysis_out: None, detections_out: None }
        }
    }

    pub fn get_session(sess_name: &String)->(FqlSession, FqlSessionUsability){
       let sess_file = BASE_DIR.to_string() + &sess_name + ".session";

       match fs::exists(&sess_file){
           Ok(true)=>{
               let sess = open_session(&sess_file, sess_name.to_string());
              
               let mut usability: FqlSessionUsability = FqlSessionUsability::NONEUSABLE;

               if !sess.meta.completed{
                  usability = FqlSessionUsability::NONEUSABLE;
               }else if sess.meta.analyzed{
                   usability = FqlSessionUsability::ANALYSISUSABLE;
               }else if sess.meta.detected{
                   usability = FqlSessionUsability::DETECTIONUSABLE;
               }else if sess.meta.framed{
                   usability = FqlSessionUsability::FRAMEUSABLE;
               }

               (sess, usability)
           },
           Ok(false)=>{
               (create_session(sess_name, false, false), FqlSessionUsability::NONEUSABLE)
           },
           Err(e)=>{
               eprintln!("Error occurred with opening file related to session. {}", e);
               (create_session(sess_name, false, false), FqlSessionUsability::NONEUSABLE)
           }
       }
    }

    //FIX ME: implement
    pub fn save_session(&mut self){
       //if not even one is completed
       if !self.meta.completed{
           self.save_metadata();
           return
       }

       self.save_detections();
       self.save_analysis();

       self.save_metadata();
    }

    pub fn clear_data(&self){
       if let Some(mut analysis) = &self.out.analysis_out{
           analysis.clear();
       }
       if let Some(mut detections) = &self.out.detections_out{
           detections.clear();
       }
    }

    fn save_metadata(&self){
       let sess_dir = BASE_DIR.to_string() + &self.name;
       let sess_file = sess_dir + ".session";

       let mut file = OpenOptions::new()
           .create(true)
           .write(true)
           .append(false)
           .open(sess_file);

       let mut flags = 0u8; 
       
       //init byte as 0 thus no need for else when a state is false
       if self.meta.cam_sourced {flags |= 1u8 << 7};
       if self.meta.user_generated {flags |= 1u8 << 6};
       if self.meta.analyzed {flags |= 1u8 << 5};
       if self.meta.detected {flags |= 1u8 << 4};
       if self.meta.framed {flags |= 1u8 << 3};
       if self.meta.completed {flags |= 1u8 << 2};

       file.write_all(&[flags]).unwrap();

       println!("session {:?} saved successfully!", self.name);
    }

    //FIX US: implement
    fn save_analysis(&mut self){
       let analysis_file = BASE_DIR.to_string() + &self.name + ".analysis";

       let file = OpenOptions::new()
           .create(true)
           .write(true)
           .append(true)
           .open(analysis_file);

       for datum in self.out.analysis_out{
           file.write_all(datum.as_bytes());
       }

       self.meta.analyzed = true;
       //for next reader to know whether anaalysis completed..
       self.meta.completed = true;
       self.out.analysis_out.clear().expect("can't clear vec for analysis");
    }

    fn save_detections(&mut self){
       let detections_file = BASE_DIR.to_string() + &self.name + ".detections";
       let file = OpenOptions::new()
           .create(true)
           .write(true)
           .append(true)
           .open(detections_file);

       for datum in self.out.detections_out{
           file.write_all(datum.as_bytes());
       }

       self.meta.detected = true;
       self.meta.completed = true;
       self.out.detections_out.clear();
    }

    #[cfg(unix)]
    pub fn save_frame(&mut self, file: &mut File, frm: &Frame)->bool{
        use std::os::unix::fs::FileExt;
        self.meta.framed = true;
        let timestamp: u64;
        if let Some(stamp) = frm.timestamp{
          timestamp = stamp.as_millis() as u64;  
          file.write_at(&timestamp.to_le_bytes(), 0);
        }else{
           timestamp = 0u64;
        }

       
        
        file.write_all(&(frm.data.len() as u64).to_le_bytes()).unwrap();
        file.write_all(&timestamp.to_le_bytes()).unwrap(); 
        file.write_all(&frm.width.to_le_bytes()).unwrap(); 
        file.write_all(&frm.height.to_le_bytes()).unwrap();
        file.write_all(&frm.data).unwrap();

        true
    }

    #[cfg(windows)]
    pub fn save_frame(&mut self, file: &mut File, cur_offset: usize, frm: &Frame)->bool{
        use std::os::windows::fs::FileExt;
        self.meta.framed = true;
        let succeeding_frm_offset = cur_offset + 64usize + 64usize + 64usize + frm.data.len();

        let timestamp: Duration;
        if let Some(stamp) = frm.timestamp{
          timestamp = stamp;
          let stamp64 = timestamp.as_millis() as u64;  
          file.write_at(&stamp64.to_le_bytes(), 0);
        }else{
           timestamp = Duration::from_millis(0);
        }


        writeln!(file, "{:?}, {:?}, {:?}, {:?}, {:?}",succeeding_frm_offset, timestamp, frm.width, frm.height, frm.data);
        true
    }

    pub fn read_analysis(&self)->io::Result<Receiver<io::Result<String>>>{
       let analysis_file = BASE_DIR.to_string() + &self.name+ ".analysis";
       let file = OpenOptions::new()
           .read(true)
           .write(false)
           .open(analysis_file);

       let (tx, rx) = sync_channel::<io::Result<String>>(5);

       let reader = BufReader::new(file);
       thread::spawn(move ||{
           for line in reader{
              let failed = line.is_err();

              if tx.send(line?).is_err() || failed{
                break;
              }
           }
       });

       Ok(rx)
    }

    pub fn read_detections(&self)->io::Result<Receiver<io::Result<String>>>{
        let detections_file = BASE_DIR.to_string() + &self.name+".detections"; 
        let file = OpenOptions::new()
           .read(true)
           .write(false)
           .open(detections_file);

        let (tx, rx) = sync_channel::<io::Result<String>>(5);
        let reader = BufReader::new(file);
        thread::spawn(move ||{
            for line in reader{
                let failed = line.is_err();

                if tx.send(line?).is_err() || failed{
                    break;
                }
            }
        });

        Ok(rx)
    }

    pub fn read_frames(&self)->FrameIter{
        let frames_file = BASE_DIR.to_string() + &self.name+".frames"; 
        let file = OpenOptions::new()
           .read(true)
           .write(false)
           .open(frames_file);

        let (tx, rx) = sync_channel::<io::Result<String>>(5);
        let reader = BufReader::new(file);
        thread::spawn(move ||{
            for line in reader{
                let failed = line.is_err();

                if tx.send(line?).is_err() || failed{
                    break;
                }
            }
        });

        FrameIter::FileSourced(rx)
    }
}
