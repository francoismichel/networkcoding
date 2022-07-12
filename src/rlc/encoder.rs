use byteorder::{BigEndian, ByteOrder};
use rand::Rng;
use rustgf::system::equation::EquationBounds;
use rustrlc::SymbolID;
use rustrlc::encoder::Encoder as RustRLCEncoder;

use crate::{EncoderError, SourceSymbolMetadata, source_symbol_metadata_to_u64, source_symbol_metadata_from_u64};
use crate::EncoderError::{BadMetadata, BufferTooSmall};

pub struct RLCEncoder {
    rust_rlc_encoder: RustRLCEncoder,
    symbol_size: usize,
    gen: tinymt::TinyMT32,
}

impl RLCEncoder {
    pub fn new(symbol_size: usize, max_window_size: usize, seed: u32) -> RLCEncoder {
        RLCEncoder{
            rust_rlc_encoder: RustRLCEncoder::new(max_window_size, symbol_size),
            gen: tinymt::TinyMT32::from_seed_u32(seed),
            symbol_size,
        }
    }

    pub fn protect_data(&mut self, data: Vec<u8>, output: &mut SourceSymbolMetadata) -> Result<usize, EncoderError> {
        if output.len() < 8 {
            return Err(BufferTooSmall);
        }
        match self.rust_rlc_encoder.protect_data(data) {
            Err(err) => {
                Err(EncoderError::InternalError(format!("{:?}", err)))
            }
            Ok(id) => {
                BigEndian::write_u64(output, id);
                Ok(8)
            }
        }
    }

    pub fn generate_and_serialize_repair_symbol_up_to(&mut self, up_to: SourceSymbolMetadata) -> Result<Vec<u8>, EncoderError> {
        let serialized_size = self.symbol_size + 8 + 8 + 4;
        let mut out = vec![0; serialized_size];
        let written = self.generate_and_serialize_repair_symbol_in_place_up_to(&mut out.as_mut_slice(), up_to)?;
        if serialized_size != written {
            Err(EncoderError::InternalError("the serialized size was not equal to prediction".to_string()))
        } else {
            Ok(out)
        }
    }


    pub fn generate_and_serialize_repair_symbol(&mut self) -> Result<Vec<u8>, EncoderError> {
        if let Some(range) = self.rust_rlc_encoder.range() {
            self.generate_and_serialize_repair_symbol_up_to(source_symbol_metadata_from_u64(*range.end()))
        } else {
            Err(EncoderError::NoSymbolToGenerate)
        }
    }

    pub fn generate_and_serialize_repair_symbol_in_place(&mut self, output: &mut [u8]) -> Result<usize, EncoderError> {
        if let Some(range) = self.rust_rlc_encoder.range() {
            self.generate_and_serialize_repair_symbol_in_place_up_to(output, source_symbol_metadata_from_u64(*range.end()))
        } else {
            Err(EncoderError::NoSymbolToGenerate)
        }
    }


    pub fn generate_and_serialize_repair_symbol_in_place_up_to(&mut self, output: &mut [u8], up_to: SourceSymbolMetadata) -> Result<usize, EncoderError> {
        if output.len() < 8 + 8 + 4 + self.symbol_size {
            return Err(BufferTooSmall);
        }
        let seed = self.gen.gen();
        let up_to = source_symbol_metadata_to_u64(up_to);
        match self.rust_rlc_encoder.generate_repair_symbol_up_to(seed, up_to) {
            Err(rustrlc::encoder::EncoderError::WindowEmpty) => Err(EncoderError::NoSymbolToGenerate),
            Err(err) => {
                Err(EncoderError::InternalError(format!("{:?}", err)))
            }
            Ok(repair_symbol) => {
                let eq = repair_symbol.to_equation();
                match eq.bounds() {
                    EquationBounds::Bounds {
                        pivot, last_nonzero_id
                    } => {
                        let mut written = 0;
                        let (pivot, last_nonzero_id) = (*pivot, *last_nonzero_id);
                        BigEndian::write_u64(&mut output[written..], pivot);
                        written += 8;
                        BigEndian::write_u64(&mut output[written..], last_nonzero_id + 1 - pivot);
                        written += 8;
                        BigEndian::write_u32(&mut output[written..], seed);
                        written += 4;

                        let data = eq.constant_term_data();
                        let len = self.symbol_size;
                        (&mut output[written..written+len]).clone_from_slice(data.as_slice());
                        written += len;
                        Ok(written)
                    }
                    EquationBounds::EmptyBounds => {
                        Err(EncoderError::NoSymbolToGenerate)
                    }
                }
            }
        }
    }

    pub fn received_symbol(&mut self, metadata: &[u8]) -> Result<usize, EncoderError> {
        if metadata.len() < 8 {
            return Err(BadMetadata);
        }
        self.rust_rlc_encoder.received_symbol(BigEndian::read_u64(metadata));
        Ok(8)
    }

    pub fn symbol_size(&self) -> usize {
        self.symbol_size
    }

    pub fn can_send_repair_symbols(&self) -> bool {
        !self.rust_rlc_encoder.is_empty()
    }

    pub fn remove_up_to(&mut self, md: SourceSymbolMetadata) {
        self.rust_rlc_encoder.remove_up_to(source_symbol_metadata_to_u64(md) as SymbolID);
    }

    pub fn next_metadata(&mut self) -> SourceSymbolMetadata {
        source_symbol_metadata_from_u64(self.rust_rlc_encoder.next_id())
    }
}