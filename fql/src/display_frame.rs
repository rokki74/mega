use opencv::{
    core::{self, Mat, Vec3b, Vec4b},
    highgui,
    imgproc,
    prelude::*,
};
use crate::frame::{Frame, PixelFormat};
use crate::detection::Detection;

fn apply_bounding(det: &Detection, mat: &mut Mat)->opencv::Result<()>{
   let width = (det.x2 - det.x1).round() as i32;
   let height = (det.y2 - det.y1).round() as i32;

   if width <=0 || height <= 0{
       return Ok(())
   }

   let x1 = det.x1.round() as i32;
   let y1 = det.x1.round() as i32;
   let rect = core::Rect::new(x1, y1, width, height);
   let color = core::Scalar::new(0.0, 0.0, 255.0, 0.0);

   let label = format!(
       "{} {:.2}",
       Detection::lookup_classname(det.class_id), 
       det.score
   );

   let text_origin = core::Point::new(x1, y1 - 5);
   let green_color = core::Scalar::new(0.0,255.0, 0.0, 0.0);

   imgproc::put_text(mat, &label, text_origin, imgproc::FONT_HERSHEY_SIMPLEX, 0.6,green_color, 2, imgproc::LINE_8, false)?;
   imgproc::rectangle(mat, rect, color, 2, imgproc::LINE_8, 0)?;

   Ok(())
}

pub fn show(dets: Vec<Detection>, frm: &Frame)->opencv::Result<()>{
  let mut mat = frame_to_bgr_mat(&frm)?; 
  
  for det in dets{
     apply_bounding(&det, &mut mat);
  }
  highgui::imshow("FQL FRAME", &mat)?;
  highgui::wait_key(1)?;

  Ok(())
}


fn frame_to_bgr_mat(frm: &Frame) -> opencv::Result<Mat> {
    let rows = i32::try_from(frm.height)
        .map_err(|_| opencv::Error::new(
            core::StsBadArg,
            "Frame height exceeds OpenCV limits"
        ))?;

    let cols = i32::try_from(frm.width)
        .map_err(|_| opencv::Error::new(
            core::StsBadArg,
            "Frame width exceeds OpenCV limits"
        ))?;

    // Determine how many bytes the frame should contain.
    let channels = match &frm.format {
        PixelFormat::Gray8 => 1,
        PixelFormat::Rgb8 | PixelFormat::Bgr8 => 3,
        PixelFormat::Rgba8 => 4,
    };

    let expected_len = frm.width
        .checked_mul(frm.height)
        .and_then(|n| n.checked_mul(channels))
        .ok_or_else(|| opencv::Error::new(
            core::StsBadArg,
            "Frame dimensions overflow"
        ))?;

    if frm.data.len() != expected_len {
        return Err(opencv::Error::new(
            core::StsBadArg,
            "Frame data length does not match dimensions and format"
        ));
    }

    let mut bgr = Mat::default();

    match &frm.format {
        PixelFormat::Bgr8 => {
            let view =
                Mat::new_rows_cols_with_bytes::<Vec3b>(
                    rows,
                    cols,
                    &frm.data,
                )?;

            bgr = view.try_clone()?;
        }

        PixelFormat::Rgb8 => {
            let rgb =
                Mat::new_rows_cols_with_bytes::<Vec3b>(
                    rows,
                    cols,
                    &frm.data,
                )?;

            // RGB -> BGR
            imgproc::cvt_color_def(
                &rgb,
                &mut bgr,
                imgproc::COLOR_RGB2BGR
            )?;
        }

        PixelFormat::Rgba8 => {
            let rgba =
                Mat::new_rows_cols_with_bytes::<Vec4b>(
                    rows,
                    cols,
                    &frm.data,
                )?;

            // RGBA -> BGR (alpha is discarded).
            imgproc::cvt_color_def(
                &rgba,
                &mut bgr,
                imgproc::COLOR_RGBA2BGR
            )?;
        }

        PixelFormat::Gray8 => {
            // Grayscale has only one channel.
            let gray =
                Mat::new_rows_cols_with_bytes::<u8>(
                    rows,
                    cols,
                    &frm.data,
                )?;

            // Gray -> BGR (replicate intensity across channels).
            imgproc::cvt_color_def(
                &gray,
                &mut bgr,
                imgproc::COLOR_GRAY2BGR
            )?;
        }
    }

    Ok(bgr)
}


