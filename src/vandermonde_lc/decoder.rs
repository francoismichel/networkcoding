use rustgf::galois_2p8;
use vandermonde_lc::{decoder::Decoder as RustVLCDecoder, SymbolID};
use vandermonde_lc::common::repair_symbol::RepairSymbol as RustVLCRepairSymbol;
use vandermonde_lc::common::source_symbol::SourceSymbol as RustVLCSourceSymbol;
use vandermonde_lc::decoder::DecoderError as VLCDecoderError;
use crate::{DecoderError, RepairSymbol, source_symbol_metadata_from_u64, SourceSymbol, SourceSymbolMetadata, source_symbol_metadata_to_u64};
use byteorder::{BigEndian, ByteOrder};
use crate::DecoderError::{BufferTooSmall};


impl From<VLCDecoderError> for DecoderError {
    fn from(err: VLCDecoderError) -> DecoderError {
        match err {
            VLCDecoderError::UnusedEquation => {
                DecoderError::UnusedRepairSymbol
            }
            VLCDecoderError::UnusedSourceSymbol => {
                DecoderError::UnusedSourceSymbol
            }
            e => DecoderError::InternalError(format!("{:?}", e))
        }
    }
}

pub struct VLCDecoder {
    rust_vlc_decoder: RustVLCDecoder,
    symbol_size: usize,
}

impl VLCDecoder {
    pub fn new(symbol_size: usize, max_window_size: usize) -> VLCDecoder {
        VLCDecoder{
            rust_vlc_decoder: RustVLCDecoder::new(symbol_size, max_window_size, Some(galois_2p8::PrimitivePolynomialField::new(galois_2p8::IrreducablePolynomial::Poly84320).unwrap())),
            symbol_size,
        }
    }

    pub fn receive_source_symbol(&mut self, source_symbol: SourceSymbol, received_at: std::time::Instant) -> Result<Vec<SourceSymbol>, DecoderError> {
        let id = BigEndian::read_u64(&source_symbol.metadata[..]);
        let recovered_ids = self.rust_vlc_decoder.add_source_symbol(RustVLCSourceSymbol::new(id, source_symbol.data), received_at)?;
        let mut ret = Vec::with_capacity(recovered_ids.len());
        for id in recovered_ids {
            ret.push(SourceSymbol{
                metadata: source_symbol_metadata_from_u64(id),
                data: self.rust_vlc_decoder.get_data(id).unwrap().clone(),
            });
        }
        Ok(ret)
    }

    pub fn read_repair_symbol(&self, data: &[u8]) -> Result<(usize, RepairSymbol), DecoderError> {
        if data.len() < 8 + 8 + 4 + self.symbol_size {
            println!("BUFFER TOO SMALL: {} VS {}", data.len(), 8 + 8 + 4 + self.symbol_size);
            return Err(BufferTooSmall);
        }
        let length = 8 + 8 + 4 + self.symbol_size;
        Ok((length, RepairSymbol{ data: data[..length].to_vec() }))
    }

    // returns (metadata_size, source_symbol)
    pub fn read_source_symbol_metadata(&self, data: &[u8]) -> Result<(usize, SourceSymbolMetadata), DecoderError> {
        if data.len() < 8 {
            println!("BUFFER TOO SMALL: {} VS {}", data.len(), 8 + 1);
            return Err(BufferTooSmall);
        }
        let id = BigEndian::read_u64(data);
        Ok((8, source_symbol_metadata_from_u64(id)))
    }

    pub fn receive_and_deserialize_repair_symbol(&mut self, repair_symbol: RepairSymbol) -> Result<(usize, Vec<SourceSymbol>), DecoderError> {
        let data = repair_symbol.data;
        if data.len() < 8 + 8 + 4 + self.symbol_size {
            return Err(BufferTooSmall);
        }

        let mut consumed = 0;
        let first_id = BigEndian::read_u64(&data[consumed..]);
        consumed += 8;
        let n_protected_symbols = BigEndian::read_u32(&data[consumed..]);
        consumed += 4;
        let sequence_number = BigEndian::read_u64(&data[consumed..]);
        consumed += 8;
        let mut symbol_data = vec![0; self.symbol_size];
        symbol_data.clone_from_slice(&data[consumed..consumed+self.symbol_size]);
        consumed += self.symbol_size;
        let recovered_ids = self.rust_vlc_decoder.add_repair_symbol(RustVLCRepairSymbol::new(first_id, sequence_number, n_protected_symbols as u64, symbol_data))?; 
        let mut ret = Vec::with_capacity(recovered_ids.len());
        for id in recovered_ids {
            ret.push(SourceSymbol{
                metadata: source_symbol_metadata_from_u64(id),
                data: self.rust_vlc_decoder.get_data(id).unwrap().clone(),
            });
        }
        Ok((consumed, ret))
    }

    pub fn symbol_size(&self) -> usize {
        self.symbol_size
    }

    pub fn remove_up_to(&mut self, md: SourceSymbolMetadata, expired_at: Option<std::time::Instant>) -> SourceSymbolMetadata {
        source_symbol_metadata_from_u64(self.rust_vlc_decoder.remove_up_to(source_symbol_metadata_to_u64(md) as SymbolID, expired_at))
    }

    pub fn bounds(&self) -> Option<(SourceSymbolMetadata, SourceSymbolMetadata)> {
        self.rust_vlc_decoder.bounds().map(|(start, end)| (source_symbol_metadata_from_u64(start), source_symbol_metadata_from_u64(end)))
    }

    pub fn largest_contiguously_received(&self) -> Option<SourceSymbolMetadata> {
        self.rust_vlc_decoder.largest_contiguously_received_id().map(|md| source_symbol_metadata_from_u64(md))
    }

    pub fn nb_missing_degrees(&self) -> Option<u64> {
        self.rust_vlc_decoder.nb_missing_degrees()
    }

    pub fn set_first_symbol_id(&mut self, md: SourceSymbolMetadata) {
        self.rust_vlc_decoder.set_first_symbol_id(source_symbol_metadata_to_u64(md));
    }
}