use crate::{frame::Frame};
use ffmpeg_next::{self as ffmpeg, format::{self}};

pub struct FfmFrame{
    pub ictx: format::context::Input,
    pub decoder: ffmpeg::codec::decoder::Video,
    pub frame: ffmpeg::frame::Video,
    pub video_stream_index: usize,
    pub time_base: ffmpeg::Rational,
}

impl Iterator for FfmFrame{
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        
        if self.decoder.receive_frame(&mut self.frame).is_ok(){
           
           return Some(Frame::from_ffmpeg(&self.frame, self.time_base));
        }


           let packet = {
               let mut packets = self.ictx.packets();

               packets.find(|(stream, _)|{
                   stream.index() == self.video_stream_index
               })
           }; 


           let Some((_,packet)) = packet else{
               return None;
           };

           let (res, received) = self.packet_sender(&packet);

           if received{
               return res;
           }

           return None;
        } 
}

impl FfmFrame{
  pub fn open(url: &String) ->FfmFrame{
    let ictx = ffmpeg::format::input(&url).unwrap();


    let input = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .unwrap();

    let video_stream_index = input.index();

    let context = ffmpeg::codec::context::Context::from_parameters(input.parameters());

    let decoder = context.unwrap().decoder().video().unwrap();
    let frame = ffmpeg::frame::Video::empty();
    let time_base = input.time_base();

    FfmFrame { ictx, decoder, frame, video_stream_index, time_base}
  }

  fn packet_sender(&mut self, packet: &ffmpeg::packet::Packet)->(Option<Frame>, bool){
        let asd = self.decoder.send_packet(packet);
        
        if let Err(ff_err) = asd {
           println!("FFMPEG ERROR SENDING DECODER PACKET: {}", ff_err);
        }

        if self.decoder.receive_frame(&mut self.frame).is_ok(){
           return (Some(Frame::from_ffmpeg(&self.frame, self.time_base)), true);
        }

        (None, false)
  }
}


