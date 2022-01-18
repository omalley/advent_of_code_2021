use std::io;
use std::io::BufRead;

use bitreader::BitReader;

fn main() {
  let stdin = io::stdin();
  for line in stdin.lock().lines()
                   .map(|x| String::from(x.unwrap().trim()))
                   .filter(|x| x.len() > 0) {
    let pack = Packet::parse(&line);
    println!("pack = {:?}, version = {}, eval = {}",
      pack, pack.version_sum(), pack.evaluation());
  }
}

#[derive(Debug)]
struct Packet {
  version: u8,
  kind: PacketKind,
}

#[derive(Debug)]
enum PacketKind {
  Sum(Vec<Packet>),
  Product(Vec<Packet>),
  Minimum(Vec<Packet>),
  Maximum(Vec<Packet>),
  Literal(u64),
  Greater(Vec<Packet>),
  Less(Vec<Packet>),
  Equal(Vec<Packet>),
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
      0 => PacketKind::Sum(Packet::parse_children(reader)),
      1 => PacketKind::Product(Packet::parse_children(reader)),
      2 => PacketKind::Minimum(Packet::parse_children(reader)),
      3 => PacketKind::Maximum(Packet::parse_children(reader)),
      4 => PacketKind::Literal(Packet::parse_literal(reader)),
      5 => PacketKind::Greater(Packet::parse_children(reader)),
      6 => PacketKind::Less(Packet::parse_children(reader)),
      7 => PacketKind::Equal(Packet::parse_children(reader)),
      _ => panic!("bad kind {}", kind_code),
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

  fn children_sum(children: &Vec<Packet>) -> u64 {
     children.iter().map(|c| c.version_sum())
                    .fold(0, |a, b| a + b)
  }

  fn version_sum(&self) -> u64 {
    self.version as u64 +
      match &self.kind {
        PacketKind::Literal(_) => 0,
        PacketKind::Sum(kids) => Packet::children_sum(kids),
        PacketKind::Product(kids) => Packet::children_sum(kids),
        PacketKind::Minimum(kids) => Packet::children_sum(kids),
        PacketKind::Maximum(kids) => Packet::children_sum(kids),
        PacketKind::Greater(kids) => Packet::children_sum(kids),
        PacketKind::Less(kids) => Packet::children_sum(kids),
        PacketKind::Equal(kids) => Packet::children_sum(kids),
      }
  }

  fn evaluation(&self) -> u64 {
    match &self.kind {
      PacketKind::Literal(lit) => *lit,
      PacketKind::Sum(kids) =>
        kids.iter().map(|k| k.evaluation()).fold(0, |a, b| a + b),
      PacketKind::Product(kids) =>
        kids.iter().map(|k| k.evaluation()).fold(1, |a, b| a * b),
      PacketKind::Minimum(kids) =>
        kids.iter().map(|k| k.evaluation())
                   .reduce(|a, b| u64::min(a, b)).unwrap(),
      PacketKind::Maximum(kids) =>
        kids.iter().map(|k| k.evaluation())
                   .reduce(|a, b| u64::max(a, b)).unwrap(),
      PacketKind::Greater(kids) =>
        kids.iter().map(|k| k.evaluation())
                   .reduce(|a, b| if a > b { 1 } else { 0 }).unwrap(),
      PacketKind::Less(kids) =>
        kids.iter().map(|k| k.evaluation())
                   .reduce(|a, b| if a < b { 1 } else { 0 }).unwrap(),
      PacketKind::Equal(kids) =>
        kids.iter().map(|k| k.evaluation())
                   .reduce(|a, b| if a == b { 1 } else { 0 }).unwrap(),
    }
  }
}
