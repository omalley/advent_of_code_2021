use std::io;
use std::io::BufRead;

use bitreader::BitReader;

fn main() {
  let stdin = io::stdin();
  for line in stdin.lock().lines()
                   .map(|x| String::from(x.unwrap().trim()))
                   .filter(|x| x.len() > 0) {
    let pack = Packet::parse(&line);
    println!("pack = {:?}, version = {}", pack, pack.version_sum());
  }
}

#[derive(Debug)]
struct Packet {
  version: u8,
  kind: PacketKind,
}

#[derive(Debug)]
enum PacketKind {
  Literal(u64),
  Operator{op: u8, children: Vec<Packet>},
}

impl Packet {
  fn parse(input: &str) -> Self {
    let vec: Vec<u8> = hex::decode(input).unwrap();
    let mut reader = BitReader::new(&vec);
    Packet::parse_packet(&mut reader)
  }

  fn parse_packet(reader: &mut BitReader) -> Self {
    let version = reader.read_u8(3).unwrap();
    let kind_code = reader.read_u8(3).unwrap();
    let kind = match kind_code {
      4 => PacketKind::Literal(Packet::parse_literal(reader)),
      _ => PacketKind::Operator{op: kind_code,
                                children: Packet::parse_children(reader)},
    };
    Packet{version: version, kind: kind}
  }

  const LITERAL_CONT_MASK: u64 = 0x10;
  
  fn parse_literal(reader: &mut BitReader) -> u64 {
    let mut result: u64 = 0;
    let mut chunks: u64 = 0;
    loop {
      let chunk = reader.read_u64(5).unwrap();
      result = (result << 4) | (chunk & !Packet::LITERAL_CONT_MASK);
      if chunk & Packet::LITERAL_CONT_MASK == 0 {
        break
      }
      chunks += 1;
      if chunks >= 16 {
        panic!("overflow in reading literal");
      }
    }
    result
  }

  fn parse_children(reader: &mut BitReader) -> Vec<Packet> {
    let mut result: Vec<Packet> = Vec::new();
    let length_type = reader.read_u8(1).unwrap();
    if length_type == 0 {
      let len = reader.read_u64(15).unwrap();
      let limit = reader.position() + len;
      while reader.position() < limit {
        result.push(Packet::parse_packet(reader));
      }
    } else {
      let kids = reader.read_u64(11).unwrap();
      for _i in 0..kids {
        result.push(Packet::parse_packet(reader));
      }
    }
    result
  }

  fn version_sum(&self) -> u64 {
    self.version as u64 +
      match &self.kind {
        PacketKind::Literal(_) => 0,
        PacketKind::Operator{op: _, children} =>
          children.iter().map(|c| c.version_sum())
                         .fold(0, |a, b| a + b),
      }
  }
}
