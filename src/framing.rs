use amqp_error::{AMQPResult, AMQPError};
use std::io::{Read, Write, Cursor};
use std::iter;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use enum_primitive::FromPrimitive;

enum_from_primitive! {
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FrameType {
    METHOD = 1,
    HEADERS = 2,
    BODY  = 3,
    HEARTBEAT = 8
}
}

impl Copy for FrameType {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Frame {
    pub frame_type: FrameType,
    pub channel: u16,
    pub payload: Vec<u8>
}

impl Frame {
    pub fn decode<T: Read>(reader: &mut T) -> AMQPResult<Frame> {
        let mut header : &mut [u8] = &mut [0u8; 7];
        try!(reader.read(&mut header));
        let mut header : &[u8] = header;
        let header : &mut &[u8] = &mut header;
        let frame_type_id = try!(header.read_u8());
        let channel = try!(header.read_u16::<BigEndian>());
        let size = try!(header.read_u32::<BigEndian>()) as usize;
        let mut payload: Vec<u8> = iter::repeat(0u8).take(size).collect();
        try!(reader.read(&mut payload));
        let frame_end = try!(reader.read_u8());
        if payload.len() != size {
            return Err(AMQPError::DecodeError("Payload didn't read the full size"));
        }
        if frame_end != 0xCE {
            return Err(AMQPError::DecodeError("Frame end wasn't right"));
        }
        let frame_type = match FrameType::from_u8(frame_type_id){
            Some(ft) => ft,
            None => return Err(AMQPError::DecodeError("Unknown frame type"))
        };

        let frame = Frame { frame_type: frame_type, channel: channel, payload : payload };
        Ok(frame)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut writer = vec!();
        writer.write_u8(self.frame_type as u8).unwrap();
        writer.write_u16::<BigEndian>(self.channel).unwrap();
        writer.write_u32::<BigEndian>(self.payload.len() as u32).unwrap();
        writer.write_all(&self.payload).unwrap();
        writer.write_u8(0xCE).unwrap();
        writer
    }
}

#[derive(Debug, Clone)]
pub struct ContentHeaderFrame {
    pub content_class: u16,
    pub weight: u16,
    pub body_size: u64,
    pub properties_flags: u16,
    pub properties: Vec<u8>
}

impl ContentHeaderFrame {
    pub fn decode(frame: Frame) -> AMQPResult<ContentHeaderFrame> {
        let mut reader = Cursor::new(frame.payload);
        let content_class = try!(reader.read_u16::<BigEndian>());
        let weight = try!(reader.read_u16::<BigEndian>()); //0 all the time for now
        let body_size = try!(reader.read_u64::<BigEndian>());
        let properties_flags = try!(reader.read_u16::<BigEndian>());
        let mut properties = vec!();
        try!(reader.read_to_end(&mut properties));
        Ok(ContentHeaderFrame {
            content_class: content_class, weight: weight, body_size: body_size,
            properties_flags: properties_flags, properties: properties
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut writer = vec!();
        writer.write_u16::<BigEndian>(self.content_class).unwrap();
        writer.write_u16::<BigEndian>(self.weight).unwrap(); //0 all the time for now
        writer.write_u64::<BigEndian>(self.body_size).unwrap();
        writer.write_u16::<BigEndian>(self.properties_flags).unwrap();
        writer.write_all(&self.properties).unwrap();
        writer
    }
}

#[test]
fn test_encode_decode(){
    let frame = Frame{ frame_type: FrameType::METHOD, channel: 5, payload: vec!(1,2,3,4,5) };
    assert_eq!(frame, Frame::decode(&mut frame.encode().as_slice()).ok().unwrap());
}
