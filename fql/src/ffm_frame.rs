use crate::{frame::Frame};
use ffmpeg_next::{self as ffmpeg, format::{self}};

use anyhow::Context;

pub struct FfmFrame{
    pub ictx: format::Input,
    pub decoder: ffmpeg::codec::decoder::Video,
    pub frame: ffmpeg::frame::Video,
    pub video_stream_index: usize,
}

impl Iterator for FfmFrame{
    type Item = Frame;
    fn next(&mut self) -> Option<Self::Item> {
        if self.decoder.receive_frame(&mut self.frame).is_ok(){
           return Some(Frame::from_ffmpeg(&self.frame));
        }

        for (stream, packet) in self.ictx.packets(){
            

            if stream.index() != self.video_stream_index{continue;}

           let (res, received) = self.packet_sender(&packet);
           if received{
               return res;
           }
        }

        return None;
    }
} 

impl FfmFrame{
  pub fn open(url: &String) ->FfmFrame{
    let mut ictx = ffmpeg::format::input(&url).unwrap();

    let input = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .unwrap();

    let video_stream_index = input.index();

    let context = ffmpeg::codec::context::Context::from_parameters(input.parameters());

    let mut decoder = context.unwrap().decoder().video().unwrap();
    let mut frame = ffmpeg::frame::Video::empty();

    FfmFrame { ictx, decoder, frame, video_stream_index}
  }

  fn packet_sender(&self, packet: &ffmpeg::packet::Packet)->(Option<Frame>, bool){
        self.decoder.send_packet(packet);
        if self.decoder.receive_frame(&mut self.frame).is_ok(){
           return (Some(Frame::from_ffmpeg(&self.frame)), true);
        }

        (None, false)
  }
}


