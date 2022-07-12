use byteorder::{BigEndian, ByteOrder};
use crate::rlc::decoder::RLCDecoder;
use crate::rlc::encoder::RLCEncoder;

pub mod rlc;

pub type SourceSymbolMetadata = [u8; 8];

#[derive(Debug)]
pub enum EncoderError {
    InternalError(String),
    BufferTooSmall,
    NoSymbolToGenerate,
    BadMetadata,
    UnImplementedEncoder,
    NoNextMetadata,
}

impl EncoderError {
    pub fn to_u64(&self) -> u64 {
        match self {
            EncoderError::InternalError(err) => {
                println!("encoder internal error: {:?}", err);
                0
            },
            EncoderError::BufferTooSmall => 1,
            EncoderError::NoSymbolToGenerate => 2,
            EncoderError::BadMetadata => 3,
            EncoderError::UnImplementedEncoder => 4,
            EncoderError::NoNextMetadata => 4,
        }
    }
}

impl DecoderError {
    pub fn to_u64(&self) -> u64 {
        match self {
            DecoderError::InternalError(err) => {
                println!("decoder internal error: {:?}", err);
                0
            },
            DecoderError::BufferTooSmall => 1,
            DecoderError::BadMetadata => 2,
            DecoderError::UnImplementedDecoder => 3,
            DecoderError::UnusedRepairSymbol => 4
        }
    }
}

#[derive(Debug)]
pub enum DecoderError {
    InternalError(String),
    BufferTooSmall,
    BadMetadata,
    UnImplementedDecoder,
    UnusedRepairSymbol,
}

#[derive(Clone, PartialEq)]
pub struct SourceSymbol {
    metadata: SourceSymbolMetadata,
    data: Vec<u8>,
}

#[derive(Clone, PartialEq)]
pub struct RepairSymbol {
    data: Vec<u8>,
}

impl RepairSymbol {
    pub fn wire_len(&self) -> usize {
        self.data.len()
    }

    pub fn take(self) -> Vec<u8> {
        self.data
    }

    pub fn get(&self) -> &Vec<u8> {
        &self.data
    }
}

impl SourceSymbol {
    pub fn new(metadata: SourceSymbolMetadata, data: Vec<u8>) -> SourceSymbol {
        SourceSymbol {
            metadata,
            data
        }
    }

    pub fn take(self) -> Vec<u8> {
        self.data
    }

    pub fn get(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn metadata(&self) -> SourceSymbolMetadata {
        self.metadata
    }
}

pub enum Encoder {
    RLC(RLCEncoder)
}

pub enum Decoder {
    RLC(RLCDecoder)
}

impl Encoder {

    ///
    /// Protects the given data and serializes its metadata into output.
    /// Returns the amount of written bytes on success
    pub fn protect_data(&mut self, data: Vec<u8>, output: &mut SourceSymbolMetadata) -> Result<usize, EncoderError> {
        match self {
            Encoder::RLC(enc) => {
                enc.protect_data(data, output)
            }
        }
    }

    ///
    /// Generates a new repair symbol protecting
    pub fn generate_and_serialize_repair_symbol_in_place_up_to(&mut self, to: &mut [u8], up_to: SourceSymbolMetadata) -> Result<usize, EncoderError> {
        match self {
            Encoder::RLC(enc) => {
                enc.generate_and_serialize_repair_symbol_in_place_up_to(to, up_to)
            }
        }
    }

    ///
    /// Generates a new repair symbol protecting
    pub fn generate_and_serialize_repair_symbol_in_place(&mut self, to: &mut [u8]) -> Result<usize, EncoderError> {
        match self {
            Encoder::RLC(enc) => {
                enc.generate_and_serialize_repair_symbol_in_place(to)
            }
        }
    }

    ///
    /// Indicates the symbol with the given metadata as received
    pub fn received_symbol(&mut self, metadata: &[u8]) -> Result<usize, EncoderError> {
        match self {
            Encoder::RLC(enc) => {
                enc.received_symbol(metadata)
            }
        }
    }

    pub fn symbol_size(&self) -> usize {
        match self {
            Encoder::RLC(enc) => {
                enc.symbol_size()
            }
        }
    }

    pub fn generate_and_serialize_repair_symbol_up_to(&mut self, up_to: SourceSymbolMetadata) -> Result<RepairSymbol, EncoderError> {
        match self {
            Encoder::RLC(enc) => {
                Ok(RepairSymbol{
                    data: enc.generate_and_serialize_repair_symbol_up_to(up_to)?,
                   })
            }
        }
    }

    pub fn generate_and_serialize_repair_symbol(&mut self) -> Result<RepairSymbol, EncoderError> {
        match self {
            Encoder::RLC(enc) => {
                Ok(RepairSymbol{
                    data: enc.generate_and_serialize_repair_symbol()?,
                   })
            }
        }
    }

    pub fn can_send_repair_symbols(&self) -> bool {
        match self {
            Encoder::RLC(enc) => {
                enc.can_send_repair_symbols()
            }
        }
    }

    pub fn remove_up_to(&mut self, md: SourceSymbolMetadata) {
        match self {
            Encoder::RLC(enc) => {
                enc.remove_up_to(md);
            }
        }
    }

    pub fn next_metadata(&mut self) -> Result<SourceSymbolMetadata, EncoderError> {
        match self {
            Encoder::RLC(enc) => {
                Ok(enc.next_metadata())
            }
            // _ => Err(EncoderError::NoNextMetadata)
        }
    }
}

impl Decoder {

    ///
    /// Protects the given data and serializes its metadata into output.
    /// Returns the amount of written bytes on success
    pub fn receive_source_symbol(&mut self, source_symbol: SourceSymbol) -> Result<Vec<SourceSymbol>, DecoderError> {
        match self {
            Decoder::RLC(dec) => {
                dec.receive_source_symbol(source_symbol)
            }
        }
    }

    ///
    /// Generates a new repair symbol protecting
    pub fn receive_and_deserialize_repair_symbol(&mut self, repair_symbol: RepairSymbol) -> Result<(usize, Vec<SourceSymbol>), DecoderError> {
        match self {
            Decoder::RLC(dec) => {
                dec.receive_and_deserialize_repair_symbol(repair_symbol)
            }
        }
    }


    pub fn read_repair_symbol(&self, data: &[u8]) -> Result<(usize, RepairSymbol), DecoderError> {
        match self {
            Decoder::RLC(dec) => {
                dec.read_repair_symbol(data)
            }
        }
    }

    pub fn read_source_symbol_metadata(&self, data: &[u8]) -> Result<(usize, SourceSymbolMetadata), DecoderError> {
        match self {
            Decoder::RLC(dec) => {
                dec.read_source_symbol_metadata(data)
            }
        }
    }

    pub fn symbol_size(&self) -> usize {
        match self {
            Decoder::RLC(dec) => {
                dec.symbol_size()
            }
        }
    }

    pub fn remove_up_to(&mut self, md: SourceSymbolMetadata) {
        match self {
            Decoder::RLC(dec) => {
                dec.remove_up_to(md);
            }
        }
    }
}

pub fn source_symbol_metadata_from_u64(n: u64) -> SourceSymbolMetadata {
    let mut ret = [0; 8];
    BigEndian::write_u64(&mut ret[..], n);
    ret
}

pub fn source_symbol_metadata_to_u64(md: SourceSymbolMetadata) -> u64  {
    BigEndian::read_u64(&md)
}
