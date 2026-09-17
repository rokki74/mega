use std::{collections::HashMap, time::Duration};

use crate::fql_compiler::parser::FqlOutCome;

#[derive(Debug, Clone)]
pub struct Detection{
    pub class_id: usize,
    pub score: f32,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub frm_no: usize,
    pub timestamp: Option<Duration>,
}

const COCO_CLASSES: [&str; 80] = [
    "person",
    "bicycle",
    "car",
    "motorcycle",
    "airplane",
    "bus",
    "train",
    "truck",
    "boat",
    "traffic light",
    "fire hydrant",
    "stop sign",
    "parking meter",
    "bench",
    "bird",
    "cat",
    "dog",
    "horse",
    "sheep",
    "cow",
    "elephant",
    "bear",
    "zebra",
    "giraffe",
    "backpack",
    "umbrella",
    "handbag",
    "tie",
    "suitcase",
    "frisbee",
    "skis",
    "snowboard",
    "sports ball",
    "kite",
    "baseball bat",
    "baseball glove",
    "skateboard",
    "surfboard",
    "tennis racket",
    "bottle",
    "wine glass",
    "cup",
    "fork",
    "knife",
    "spoon",
    "bowl",
    "banana",
    "apple",
    "sandwich",
    "orange",
    "broccoli",
    "carrot",
    "hot dog",
    "pizza",
    "donut",
    "cake",
    "chair",
    "couch",
    "potted plant",
    "bed",
    "dining table",
    "toilet",
    "tv",
    "laptop",
    "mouse",
    "remote",
    "keyboard",
    "cell phone",
    "microwave",
    "oven",
    "toaster",
    "sink",
    "refrigerator",
    "book",
    "clock",
    "vase",
    "scissors",
    "teddy bear",
    "hair drier",
    "toothbrush",
];

pub fn detect_show(d: &Detection){
       println!("{} {:.2} ({}, {}, {}, {})",
       COCO_CLASSES[d.class_id],
       d.score,
       d.x1,
       d.y1,
       d.x2,
       d.y2
       );

       let frm_no = d.frm_no;

       match d.timestamp{
           Some(timestamp) =>{
              match d.class_id{     
                   0 => println!("PERSON detected on timestamp: {:#?} for frame number: {}", timestamp, frm_no),
                   2 |3 | 5| 7 =>println!("VEHICLE detected on timestamp: {:#?} for frame number: {}", timestamp, frm_no),
                   16..=21 =>println!("ANIMAL detected on timestamp: {:#?} for frame number: {}", timestamp, frm_no),
                   _=>{}
               }
           },
           None =>{
              match d.class_id{
                   0 => println!("PERSON detected on timestamp: {:#?} for frame number: {}", "No Available timestamp", frm_no),
                   2 |3 | 5| 7 =>println!("VEHICLE detected on timestamp: {:#?} for frame number: {}", "No Available timestamp", frm_no),
                   16..=21 =>println!("ANIMAL detected on timestamp: {:#?} for frame number: {}", "No Available timestamp", frm_no),
                   _=>{}
              }
           }
       }
}

pub fn detect_all_show(detections: &Vec<Detection>){
   for d in detections{
      detect_show(&d); 
   }
}

enum OutCome{
    NULL,
    Res(String),
}

#[inline]
fn detect(d: &Detection)->OutCome{
       let res_string = String::new();
       println!("{} {:.2} ({}, {}, {}, {})",
       COCO_CLASSES[d.class_id],
       d.score,
       d.x1,
       d.y1,
       d.x2,
       d.y2
       );

       let frm_no = d.frm_no;

       match d.timestamp{
           Some(timestamp) =>{
              match d.class_id{     
                   0 => {
                       let f = format!("PERSON detected on timestamp: {:#?} for frame number: {}", timestamp, frm_no);
                       return OutCome::Res(f);
                   },
                   2 |3 | 5| 7 =>{
                       let f = format!("VEHICLE detected on timestamp: {:#?} for frame number: {}", timestamp, frm_no);
                       return OutCome::Res(f);
                    },
                   16..=21 =>{ let f = format!("ANIMAL detected on timestamp: {:#?} for frame number: {}", timestamp, frm_no);
                       return OutCome::Res(f);
                   },
                   _=>{
                       return OutCome::NULL;
                   }
               }
           },
           None =>{
              match d.class_id{
                   0 => { let f = format!("PERSON detected on timestamp: {:#?} for frame number: {}", "No Available timestamp", frm_no);
                      return OutCome::Res(f);
                   },
                   2 |3 | 5| 7 =>println!("VEHICLE detected on timestamp: {:#?} for frame number: {}", "No Available timestamp", frm_no),
                   16..=21 =>{ let f = format!("ANIMAL detected on timestamp: {:#?} for frame number: {}", "No Available timestamp", frm_no);
                     return OutCome::Res(f);
                   },
                   _=>{
                       return OutCome::NULL;
                   }
              }
           }
       }

       OutCome::Res(res_string)
}

pub fn detect_back(results: Vec<Detection>)->Option<FqlOutCome>{
    let mut out: Vec<String> = Vec::new();
    for res in results{
        let dete = detect(&res);
        if let OutCome::Res(r) = dete{
           out.push(r) 
        }
    }

    if out.is_empty(){
        return None;
    }
    
    Some(FqlOutCome::Multiple(out))
}

pub fn iou(det: &Detection, other: Detection) ->f32{
    let d_width = det.x2 - det.x1;
    let d_height = det.y2 - det.y1;
    let (d_left, d_right, d_top, d_bottom) =(det.x1, (det.x1 + d_width), det.y2, (det.y1 + d_height));


   let o_width = other.x2 - other.x1;
   let o_height = other.y2 - other.y1;
   let (o_left, o_right, o_top, o_bottom) =(other.x1, (other.x1 + o_width), other.y2, (other.y1 + o_height));

   //unions 
   let left = d_left.max(o_left);
   let right = d_right.min(o_right);
   let top = d_top.max(o_top);
   let bottom = d_bottom.min(o_bottom);

   if left <= right || bottom <= top{
       return 0.0;
   }
   
   let w = right - left;
   let h = bottom - top;

   let intersection = w * h;

   let area_d = d_width * d_height;
   let area_o = o_width * o_height;

   let union = area_d + area_o - intersection;
   intersection/union
}

impl Detection{
    pub fn match_detections(det1: &Detection, det2: &Detection)->bool{
       if det1.class_id == det2.class_id{
           return true;
       }

       false
    }

    pub fn lookup_classname(classid: usize)->String{
        COCO_CLASSES[classid].to_string()
    }

    pub fn fill_coco_classes_map()->HashMap<String, usize>{
        let coco: HashMap<String, usize> = HashMap::new();

        for (datum, i) in COCO_CLASSES.into_iter().enumerate(){
            coco.insert(datum.to_string(), i);
        }

        coco
    }
}
