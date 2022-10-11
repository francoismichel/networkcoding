use core::slice;
use std::mem;
use libc::size_t;
use libc::ssize_t;

use crate::Decoder;
use crate::RepairSymbol;
use crate::SourceSymbol;
use crate::source_symbol_metadata_from_u64;
use crate::source_symbol_metadata_to_u64;
use crate::vandermonde_lc::decoder::VLCDecoder;
use crate::vandermonde_lc::encoder::VLCEncoder;
use crate::{Encoder, SourceSymbolMetadata};


#[allow(non_camel_case_types)]
type source_symbol_metadata_t = u64;
#[allow(non_camel_case_types)]
type encoder_t = Encoder;
#[allow(non_camel_case_types)]
type decoder_t = Decoder;

pub struct SourceSymbolsBuffer {
    current_index: size_t,
    symbols: Vec<SourceSymbol>, // we use a Vec and not VecDeque to avoid doing reallocations to go from Vec to VecDeque in decoder functions
}

#[allow(non_camel_case_types)]
type source_symbols_buffer_t = SourceSymbolsBuffer;

#[no_mangle]
pub extern "C" fn new_vlc_encoder(symbol_size: size_t, window_size: size_t) -> *mut encoder_t {
    let vlc_encoder = VLCEncoder::new(symbol_size, window_size);
    Box::into_raw(Box::new(Encoder::VLC(vlc_encoder)))
}


#[no_mangle]
pub extern "C" fn destroy_encoder(encoder: *mut encoder_t) {
    unsafe { Box::from_raw(encoder) };
}


#[no_mangle]
pub extern "C" fn new_vlc_decoder(symbol_size: size_t, window_size: size_t) -> *mut decoder_t {
    let vlc_decoder = VLCDecoder::new(symbol_size, window_size);
    Box::into_raw(Box::new(Decoder::VLC(vlc_decoder)))
}


#[no_mangle]
pub extern "C" fn destroy_decoder(decoder: *mut decoder_t) {
    unsafe { Box::from_raw(decoder) };
}


/// Encoder-specific functions


///
/// Protects the given data and serializes its metadata into output.
/// Returns the amount of written bytes on success
#[no_mangle]
pub extern "C" fn encoder_protect_data(encoder: &mut encoder_t, data: *mut u8, data_len: size_t, output: *mut source_symbol_metadata_t) -> ssize_t {

    let buf = unsafe { slice::from_raw_parts(data, data_len) };
    let mut md = source_symbol_metadata_from_u64(0);
    match encoder.protect_data(buf.to_vec(), &mut md) {
        Ok(v) =>  {
            unsafe {*output = source_symbol_metadata_to_u64(md)}
            v as ssize_t
        }
        Err(e) => e.to_c(),
    }
}

///
/// Generates a new repair symbol protecting
#[no_mangle]
pub extern "C" fn encoder_generate_and_serialize_repair_symbol_up_to(encoder: &mut encoder_t, out: *mut u8, out_len: size_t, up_to: source_symbol_metadata_t) -> ssize_t {
    let buf = unsafe { slice::from_raw_parts_mut(out, out_len) };
    match encoder.generate_and_serialize_repair_symbol_in_place_up_to(buf, source_symbol_metadata_from_u64(up_to))  {
        Ok(v) => v as ssize_t,
        Err(e) => e.to_c(),
    }
}

///
/// Generates a new repair symbol protecting
#[no_mangle]
pub extern "C" fn encoder_generate_and_serialize_repair_symbol(encoder: &mut encoder_t, out: *mut u8, out_len: size_t) -> ssize_t {
    let buf = unsafe { slice::from_raw_parts_mut(out, out_len) };
    match encoder.generate_and_serialize_repair_symbol_in_place(buf)  {
        Ok(v) => v as ssize_t,
        Err(e) => e.to_c(),
    }
}

///
/// Indicates the symbol with the given metadata as received
#[no_mangle]
pub extern "C" fn encoder_received_symbol(encoder: &mut encoder_t, metadata: *const u8, len: size_t) -> ssize_t {
    let buf = unsafe { slice::from_raw_parts(metadata, len) };
    match encoder.received_symbol(buf)  {
        Ok(v) => v as ssize_t,
        Err(e) => e.to_c(),
    }
}

#[no_mangle]
pub extern "C" fn encoder_symbol_size(encoder: &mut encoder_t) -> size_t {
    encoder.symbol_size()
}

#[no_mangle]
pub extern "C" fn encoder_can_send_repair_symbols(encoder: &mut encoder_t) -> bool {
    encoder.can_send_repair_symbols()
}

#[no_mangle]
pub extern "C" fn encoder_remove_up_to(encoder: &mut encoder_t, up_to: source_symbol_metadata_t) {
    encoder.remove_up_to(source_symbol_metadata_from_u64(up_to))
}

#[no_mangle]
pub extern "C" fn encoder_next_metadata(encoder: &mut encoder_t, out: *mut u8, len: size_t ) -> ssize_t {
    if len < mem::size_of::<SourceSymbolMetadata>() {
        panic!("source_symbol_metadata buffer too small");
    }
    match encoder.next_metadata() {
        Ok(v) => {
            let buf = unsafe { slice::from_raw_parts_mut(out, len) };
            buf.copy_from_slice(&v[..]);
            0
        }
        Err(e) => e.to_c(),
    }
}

fn new_source_symbols_buffer(data: Vec<SourceSymbol>) -> source_symbols_buffer_t {
    source_symbols_buffer_t { current_index: 0, symbols: data }
}

#[no_mangle]
pub extern "C" fn source_symbols_buffer_dequeue(buffer: &mut source_symbols_buffer_t, out: *mut u8, out_len: size_t, out_metadata: &mut source_symbol_metadata_t) -> ssize_t {
    if buffer.current_index >= buffer.symbols.len() {
        return -1 as isize;
    }
    let out_symbol = &buffer.symbols[buffer.current_index];
    if out_len < out_symbol.data.len() {
        return -2 as isize;
    }
    let buf = unsafe { slice::from_raw_parts_mut(out, out_len) };
    buf.copy_from_slice(&out_symbol.data[..]);
    *out_metadata = source_symbol_metadata_to_u64(out_symbol.metadata());
    buffer.current_index += 1;
    out_symbol.data.len() as ssize_t
}

#[no_mangle]
pub extern "C" fn source_symbols_buffer_is_empty(buffer: &mut source_symbols_buffer_t) -> bool {
    buffer.current_index >= buffer.symbols.len()
}


// #[no_mangle]
// pub extern "C" fn destroy_decoder(decoder: *mut decoder_t) {
//     unsafe { Box::from_raw(decoder) };
// }
#[no_mangle]
pub extern "C" fn destroy_source_symbols_buffer(buffer: *mut source_symbols_buffer_t) {
    unsafe { drop(Box::from_raw(buffer)); }
}



/// Decoder-specific functions


///
/// the given source_sylmbol_data is copied
#[no_mangle]
pub extern "C" fn decoder_receive_source_symbol(decoder: &mut decoder_t, metadata: source_symbol_metadata_t, source_symbol_data: *mut u8, len: size_t) -> *mut source_symbols_buffer_t {
    if len != decoder.symbol_size() {
        panic!("source_symbol_data different from decoder's symbol size");
    }
    let buf = unsafe { slice::from_raw_parts_mut(source_symbol_data, len) };
    let source_symbol = SourceSymbol::new(source_symbol_metadata_from_u64(metadata), Vec::from(buf));

    match decoder.receive_source_symbol(source_symbol) {
        Ok(recovered) => Box::into_raw(Box::new(new_source_symbols_buffer(recovered))),
        Err(_) => std::ptr::null_mut(),
    }
}

///
/// Generates a new repair symbol protecting
#[no_mangle]
pub extern "C" fn decoder_receive_and_deserialize_repair_symbol(decoder: &mut decoder_t, repair_symbol_data: *mut u8, len: size_t, consumed: *mut size_t) -> *mut source_symbols_buffer_t {
    
    // let repair_symbol = RepairSymbol::new(source_symbol_metadata_from_u64(metadata), Vec::from(buf));
    let repair_symbol = RepairSymbol{
        data: unsafe { slice::from_raw_parts_mut(repair_symbol_data, len) }.to_vec(),
    };
    match decoder.receive_and_deserialize_repair_symbol(repair_symbol) {
        Ok((cons, recovered)) => {
            unsafe { *consumed = cons };
            Box::into_raw(Box::new(new_source_symbols_buffer(recovered)))
        }
        Err(_) => std::ptr::null_mut()
    }
}


/// reads the payload and tells the length of the repair symbol including the symbol size + potential metadata
#[no_mangle]
pub extern "C" fn decoder_get_repair_symbol_payload_length(decoder: &decoder_t, data: *mut u8, total_len: size_t) -> ssize_t {
    let buf = unsafe { slice::from_raw_parts_mut(data, total_len) };
    match decoder.read_repair_symbol(buf) {
        Ok((consumed, _)) => {
            consumed as isize
        }
        Err(e) => e.to_c()
    }
}

#[no_mangle]
pub extern "C" fn decoder_read_source_symbol_metadata(decoder: &decoder_t, data: *mut u8, len: size_t, out: *mut source_symbol_metadata_t) -> ssize_t {
    let buf = unsafe { slice::from_raw_parts_mut(data, len) };

    match decoder.read_source_symbol_metadata(buf) {
        Ok((consumed, metadata)) => {
            unsafe { *out = source_symbol_metadata_to_u64(metadata) };
            consumed as isize
        }
        Err(e) => e.to_c()
    }
}

#[no_mangle]
pub extern "C" fn decoder_symbol_size(decoder: &decoder_t) -> size_t {
    match decoder {
        #[cfg(feature = "enable-rlc")]
        Decoder::RLC(dec) => {
            dec.symbol_size()
        }
        Decoder::VLC(dec) => {
            dec.symbol_size()
        }
    }
}

#[no_mangle]
pub extern "C" fn decoder_remove_up_to(decoder: &mut decoder_t, md: source_symbol_metadata_t) {
    decoder.remove_up_to(source_symbol_metadata_from_u64(md))
}


