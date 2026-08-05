#[derive(Debug)]
pub struct Detection{
    class_id: usize,
    score: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

fn detect(mut detections: &Vec<Detection>){
    todo!();
}

pub fn iou(det: &Detection, other: Detection) ->f32{
    let d_width = det.x2 - det.x1;
    let d_height = det.y2 - det.y1;
    let (d_left, d_right, d_top, d_bottom) =(det.x, (det.x + d_width), det.y, (det.y + d_height));

   let o_width = other.x2 - other.x1;
   let o_height = other.y2 - other.y1;
   let (o_left, o_right, o_top, o_bottom) =(other.x, (other.x + o_width), other.y, (other.y + o_height));

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
