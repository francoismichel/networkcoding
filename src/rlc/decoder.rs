use rustrlc::{decoder::Decoder as RustRLCDecoder, SymbolID};
use rustrlc::common::repair_symbol::RepairSymbol as RustRLCRepairSymbol;
use rustrlc::common::source_symbol::SourceSymbol as RustRLCSourceSymbol;
use crate::{DecoderError, RepairSymbol, source_symbol_metadata_from_u64, SourceSymbol, SourceSymbolMetadata, source_symbol_metadata_to_u64};
use byteorder::{BigEndian, ByteOrder};
use crate::DecoderError::{BufferTooSmall, InternalError};

pub struct RLCDecoder {
    rust_rlc_decoder: RustRLCDecoder,
    symbol_size: usize,
}

impl RLCDecoder {
    pub fn new(symbol_size: usize, max_window_size: usize) -> RLCDecoder {
        RLCDecoder{
            rust_rlc_decoder: RustRLCDecoder::new(symbol_size, max_window_size),
            symbol_size,
        }
    }

    pub fn receive_source_symbol(&mut self, source_symbol: SourceSymbol) -> Result<Vec<SourceSymbol>, DecoderError> {
        let id = BigEndian::read_u64(&source_symbol.metadata[..]);
        match self.rust_rlc_decoder.add_source_symbol(RustRLCSourceSymbol::new(id, source_symbol.data)) {
            Err(err) => {
                return Err(InternalError(format!("{:?}", err)));
            }
            Ok(recovered_ids) => {
                let mut ret = Vec::with_capacity(recovered_ids.len());
                for id in recovered_ids {
                    ret.push(SourceSymbol{
                        metadata: source_symbol_metadata_from_u64(id),
                        data: self.rust_rlc_decoder.get_data(id).unwrap().clone(),
                    });
                }
                Ok(ret)
            }
        }
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
        let n_protected_symbols = BigEndian::read_u64(&data[consumed..]);
        consumed += 8;
        let seed = BigEndian::read_u32(&data[consumed..]);
        consumed += 4;
        let mut symbol_data = vec![0; self.symbol_size];
        symbol_data.clone_from_slice(&data[consumed..consumed+self.symbol_size]);
        consumed += self.symbol_size;
        match self.rust_rlc_decoder.add_repair_symbol(RustRLCRepairSymbol::new(seed, first_id, n_protected_symbols, symbol_data)) {
            Err(err) => {
                return match err {
                    rustrlc::decoder::DecoderError::UnusedEquation => Err(DecoderError::UnusedRepairSymbol),
                    _ => Err(InternalError(format!("{:?}", err))),
                }
            }
            Ok(recovered_ids) => {
                let mut ret = Vec::with_capacity(recovered_ids.len());
                for id in recovered_ids {
                    ret.push(SourceSymbol{
                        metadata: source_symbol_metadata_from_u64(id),
                        data: self.rust_rlc_decoder.get_data(id).unwrap().clone(),
                    });
                }
                Ok((consumed, ret))
            }
        }
    }

    pub fn symbol_size(&self) -> usize {
        self.symbol_size
    }

    pub fn remove_up_to(&mut self, md: SourceSymbolMetadata) {
        self.rust_rlc_decoder.remove_up_to(source_symbol_metadata_to_u64(md) as SymbolID);
    }

    pub fn bounds(&self) -> Option<(SourceSymbolMetadata, SourceSymbolMetadata)> {
        self.rust_rlc_decoder.bounds().map(|(start, end)| (source_symbol_metadata_from_u64(start), source_symbol_metadata_from_u64(end)))
    }
}