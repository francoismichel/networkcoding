use byteorder::{BigEndian, ByteOrder};
use rustgf::galois_2p8;
use rustgf::system::equation::EquationBounds;
use vandermonde_lc::SymbolID;
use vandermonde_lc::encoder::Encoder as RustVLCEncoder;

use crate::{EncoderError, SourceSymbolMetadata, source_symbol_metadata_to_u64, source_symbol_metadata_from_u64};
use crate::EncoderError::{BadMetadata, BufferTooSmall};

pub struct VLCEncoder {
    rust_vlc_encoder: RustVLCEncoder,
    symbol_size: usize,
}

impl VLCEncoder {
    pub fn new(symbol_size: usize, max_window_size: usize) -> VLCEncoder {
        VLCEncoder{
            rust_vlc_encoder: RustVLCEncoder::new(max_window_size, symbol_size, Some(galois_2p8::PrimitivePolynomialField::new(galois_2p8::IrreducablePolynomial::Poly84320).unwrap())),
            symbol_size,
        }
    }

    pub fn protect_data(&mut self, data: Vec<u8>, output: &mut SourceSymbolMetadata) -> Result<usize, EncoderError> {
        if output.len() < 8 {
            return Err(BufferTooSmall);
        }
        match self.rust_vlc_encoder.protect_data(data) {
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
        let written = self.generate_and_serialize_repair_symbol_in_place_up_to(out.as_mut_slice(), up_to)?;
        if serialized_size != written {
            Err(EncoderError::InternalError("the serialized size was not equal to prediction".to_string()))
        } else {
            Ok(out)
        }
    }


    pub fn generate_and_serialize_repair_symbol(&mut self) -> Result<Vec<u8>, EncoderError> {
        if let Some(range) = self.rust_vlc_encoder.range() {
            self.generate_and_serialize_repair_symbol_up_to(source_symbol_metadata_from_u64(*range.end()))
        } else {
            Err(EncoderError::NoSymbolToGenerate)
        }
    }

    pub fn generate_and_serialize_repair_symbol_in_place(&mut self, output: &mut [u8]) -> Result<usize, EncoderError> {
        if let Some(range) = self.rust_vlc_encoder.range() {
            self.generate_and_serialize_repair_symbol_in_place_up_to(output, source_symbol_metadata_from_u64(*range.end()))
        } else {
            Err(EncoderError::NoSymbolToGenerate)
        }
    }


    pub fn generate_and_serialize_repair_symbol_in_place_up_to(&mut self, output: &mut [u8], up_to: SourceSymbolMetadata) -> Result<usize, EncoderError> {
        if output.len() < 8 + 8 + 4 + self.symbol_size {
            return Err(BufferTooSmall);
        }
        let up_to = source_symbol_metadata_to_u64(up_to);
        match self.rust_vlc_encoder.generate_repair_symbol_up_to(up_to) {
            Err(vandermonde_lc::encoder::EncoderError::WindowEmpty) => Err(EncoderError::NoSymbolToGenerate),
            Err(err) => {
                Err(EncoderError::InternalError(format!("{:?}", err)))
            }
            Ok(repair_symbol) => {
                let sequence_number = repair_symbol.sequence_number();
                let eq = repair_symbol.to_equation();
                match eq.bounds() {
                    EquationBounds::Bounds {
                        pivot, last_nonzero_id
                    } => {
                        let mut written = 0;
                        let (pivot, last_nonzero_id) = (*pivot, *last_nonzero_id);
                        BigEndian::write_u64(&mut output[written..], pivot);
                        written += 8;
                        BigEndian::write_u32(&mut output[written..], (last_nonzero_id + 1 - pivot) as u32);
                        written += 4;
                        BigEndian::write_u64(&mut output[written..], sequence_number);
                        written += 8;

                        let data = eq.constant_term_data();
                        let len = self.symbol_size;
                        output[written..written+len].clone_from_slice(data.as_slice());
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
        self.rust_vlc_encoder.received_symbol(BigEndian::read_u64(metadata));
        Ok(8)
    }

    pub fn symbol_size(&self) -> usize {
        self.symbol_size
    }

    pub fn can_send_repair_symbols(&self) -> bool {
        !self.rust_vlc_encoder.is_empty()
    }

    pub fn remove_up_to(&mut self, md: SourceSymbolMetadata) {
        self.rust_vlc_encoder.remove_up_to(source_symbol_metadata_to_u64(md) as SymbolID);
    }

    pub fn next_metadata(&mut self) -> SourceSymbolMetadata {
        source_symbol_metadata_from_u64(self.rust_vlc_encoder.next_id())
    }

    pub fn next_repair_symbol_size(&self, _up_to: SourceSymbolMetadata) -> usize {
        self.symbol_size + 8 + 8 + 4
    }   
    
    pub fn first_metadata(&self) -> Option<SourceSymbolMetadata> {
        self.rust_vlc_encoder.range().map(|range| source_symbol_metadata_from_u64(*range.start()))
    }
    
    pub fn last_metadata(&self) -> Option<SourceSymbolMetadata> {
        self.rust_vlc_encoder.range().map(|range| source_symbol_metadata_from_u64(*range.end()))
    }
    
    pub fn current_window_size(&self) -> usize {
        match self.rust_vlc_encoder.range() {
            None => 0,
            Some(range) => (range.end() + 1 - range.start()) as usize,
        }
    }

    pub fn get_sent_time(&self, md: SourceSymbolMetadata) -> Option<std::time::Instant> {
        self.rust_vlc_encoder.get_sent_time(source_symbol_metadata_to_u64(md))
    }
}