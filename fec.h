#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct Decoder Decoder;

typedef struct Encoder Encoder;

typedef struct SourceSymbolsBuffer SourceSymbolsBuffer;

typedef struct Encoder encoder_t;

typedef struct Decoder decoder_t;

typedef uint64_t source_symbol_metadata_t;

typedef struct SourceSymbolsBuffer source_symbols_buffer_t;

encoder_t *new_vlc_encoder(size_t symbol_size, size_t window_size);

void destroy_encoder(encoder_t *encoder);

decoder_t *new_vlc_decoder(size_t symbol_size, size_t window_size);

void destroy_decoder(decoder_t *decoder);

/**
 * Encoder-specific functions
 *
 * Protects the given data and serializes its metadata into output.
 * Returns the amount of written bytes on success
 */
ssize_t encoder_protect_data(encoder_t *encoder,
                             uint8_t *data,
                             size_t data_len,
                             source_symbol_metadata_t *output);

/**
 *
 * Generates a new repair symbol protecting
 */
ssize_t encoder_generate_and_serialize_repair_symbol_up_to(encoder_t *encoder,
                                                           uint8_t *out,
                                                           size_t out_len,
                                                           source_symbol_metadata_t up_to);

/**
 *
 * Generates a new repair symbol protecting
 */
ssize_t encoder_generate_and_serialize_repair_symbol(encoder_t *encoder,
                                                     uint8_t *out,
                                                     size_t out_len);

/**
 *
 * Indicates the symbol with the given metadata as received
 */
ssize_t encoder_received_symbol(encoder_t *encoder, const uint8_t *metadata, size_t len);

size_t encoder_symbol_size(encoder_t *encoder);

bool encoder_can_send_repair_symbols(encoder_t *encoder);

void encoder_remove_up_to(encoder_t *encoder, source_symbol_metadata_t up_to);

ssize_t encoder_next_metadata(encoder_t *encoder, uint8_t *out, size_t len);

ssize_t source_symbols_buffer_dequeue(source_symbols_buffer_t *buffer,
                                      uint8_t *out,
                                      size_t out_len,
                                      source_symbol_metadata_t *out_metadata);

bool source_symbols_buffer_is_empty(source_symbols_buffer_t *buffer);

void destroy_source_symbols_buffer(source_symbols_buffer_t *buffer);

/**
 * Decoder-specific functions
 *
 * the given source_sylmbol_data is copied
 */
source_symbols_buffer_t *decoder_receive_source_symbol(decoder_t *decoder,
                                                       source_symbol_metadata_t metadata,
                                                       uint8_t *source_symbol_data,
                                                       size_t len);

/**
 *
 * Generates a new repair symbol protecting
 */
source_symbols_buffer_t *decoder_receive_and_deserialize_repair_symbol(decoder_t *decoder,
                                                                       uint8_t *repair_symbol_data,
                                                                       size_t len,
                                                                       size_t *consumed);

/**
 * reads the payload and tells the length of the repair symbol including the symbol size + potential metadata
 */
ssize_t decoder_get_repair_symbol_payload_length(const decoder_t *decoder,
                                                 uint8_t *data,
                                                 size_t total_len);

ssize_t decoder_read_source_symbol_metadata(const decoder_t *decoder,
                                            uint8_t *data,
                                            size_t len,
                                            source_symbol_metadata_t *out);

size_t decoder_symbol_size(const decoder_t *decoder);

void decoder_remove_up_to(decoder_t *decoder, source_symbol_metadata_t md);
