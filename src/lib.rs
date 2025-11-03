#![allow(warnings)]
use std::fs::read;

#[derive(Debug)]
struct PNG {
    signature: [u8; 8],
    ihdr: IHDR,
    idat: Vec<IDAT>,
    plte: Option<PLTE>,
    optional_chunks: Vec<OptionalChunk>,
}

#[derive(Debug)]
enum OptionalChunk {}

#[derive(Clone, Debug)]
struct IHDR {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

#[derive(Debug)]
enum IDAT {
    IDAT_RGBA(IDAT_RGBA),
    IDAT_Pallet(IDAT_Pallet),
}

#[derive(Debug)]
struct Scanline {
    filter_method: u8,
    pixels: Vec<u8>,
}

#[derive(Debug)]
struct IDAT_Pallet {
    scanlines: Vec<Scanline>,
}

#[derive(Debug)]
struct IDAT_RGBA {
    pixels: Vec<Scanline>,
}

#[derive(Debug)]
struct PLTE {
    pallet: Vec<(u8, u8, u8)>,
}

fn read_chunks(bytes: &[u8]) -> PNG {
    let signature: [u8; 8] = bytes[0..8].try_into().unwrap();
    let mut ihdr: Option<IHDR> = None;
    let mut idat: Vec<IDAT> = Vec::<IDAT>::new();
    let mut plte: Option<PLTE> = None;
    let mut optional_chunks = Vec::<OptionalChunk>::new();

    let mut index = 8;
    while index < bytes.len() {
        let chunk_length = u32::from_be_bytes(bytes[index..index + 4].try_into().unwrap()) as usize;
        index += 4;
        let chunk_type = std::str::from_utf8(&bytes[index..index + 4]).unwrap();
        index += 4;
        let chunk_data = &bytes[index..index + chunk_length];
        index += chunk_length;
        let _crc = u32::from_be_bytes(bytes[index..index + 4].try_into().unwrap());
        index += 4;

        match chunk_type {
            "IHDR" => {
                ihdr = Some(IHDR {
                    width: u32::from_be_bytes(chunk_data[0..4].try_into().unwrap()),
                    height: u32::from_be_bytes(chunk_data[4..8].try_into().unwrap()),
                    bit_depth: chunk_data[8],
                    color_type: chunk_data[9],
                    compression_method: chunk_data[10],
                    filter_method: chunk_data[11],
                    interlace_method: chunk_data[12],
                });
            }
            "IDAT" => {
                let ihdr = ihdr.as_ref().expect("IHDR is not present!");

                if ihdr.filter_method != 0 {
                    panic!("Unsupported filter method: {}", ihdr.filter_method);
                }
                if ihdr.color_type != 3 {
                    panic!("Unsupported color type: {}", ihdr.color_type);
                }

                let mut index = 0;
                let zlib_header = &chunk_data[index..index + 2];
                let mut scanlines = Vec::<Scanline>::new();
                index += 2;
                while index < chunk_data.len() {
                    let deflate_header = chunk_data[index];
                    index += 1;
                    let is_last_block = (deflate_header & 0b0000_0001) == 1;
                    let compressiton = (deflate_header & 0b0000_0110);
                    if compressiton != 0b0000_0000 {
                        panic!("Unsupported compression method: {:b}", compressiton);
                    }
                    let len: u16 =
                        u16::from_le_bytes(chunk_data[index..index + 2].try_into().unwrap());
                    index += 2;
                    let len_complement =
                        u16::from_le_bytes(chunk_data[index..index + 2].try_into().unwrap());
                    index += 2;

                    let block_data = &chunk_data[index..index + len as usize];
                    index += len as usize;

                    let mut index = 0;
                    while index < block_data.len() {
                        let filter_method = block_data[index];
                        index += 1;
                        let scanline_size_bytes =
                            ihdr.width.div_ceil((8 / ihdr.bit_depth) as u32) as usize;
                        let pixels = block_data[index..index + scanline_size_bytes].to_vec();
                        let scanline = Scanline {
                            filter_method: filter_method,
                            pixels: pixels,
                        };
                        scanlines.push(scanline);
                        index += scanline_size_bytes;
                    }

                    if is_last_block {
                        break;
                    }
                }

                if ihdr.color_type == 3 {
                    idat.push(IDAT::IDAT_Pallet(IDAT_Pallet {
                        scanlines: scanlines,
                    }));
                }
            }
            "PLTE" => {
                let mut pallet = Vec::<(u8, u8, u8)>::new();

                let mut index = 0;
                while index < chunk_data.len() {
                    let r = chunk_data[index];
                    index += 1;
                    let g = chunk_data[index];
                    index += 1;
                    let b = chunk_data[index];
                    index += 1;
                    pallet.push((r, g, b));
                }

                plte = Some(PLTE { pallet: pallet })
            }
            "IEND" => break,
            _ => {
                if chunk_type.chars().next().unwrap().is_ascii_uppercase() {
                    panic!("Unsupported critical chunk: {}", chunk_type);
                }
            }
        }
    }

    PNG {
        signature: signature,
        ihdr: ihdr.expect("IHDR is not present!"),
        idat: idat,
        plte: plte,
        optional_chunks: optional_chunks,
    }
}

pub fn read_png(path: &str) -> Result<Vec<Vec<(u8, u8, u8)>>, &str> {
    let file_bytes = read(path).unwrap();

    let png = read_chunks(&file_bytes);

    let mut colors = Vec::<Vec<(u8, u8, u8)>>::new();

    for idat in &png.idat {
        match idat {
            IDAT::IDAT_Pallet(idat) => {
                for scanline in &idat.scanlines {
                    let mut x = 0;
                    let mut row = Vec::<(u8, u8, u8)>::new();
                    let mut current_bit = 0;
                    'scanline: for byte in &scanline.pixels {
                        while current_bit < 8 {
                            let pallet_color_index =
                                ((byte << current_bit) & (255 << (8 - png.ihdr.bit_depth))) >> 7;
                            let pallet_color =
                                png.plte.as_ref().unwrap().pallet[pallet_color_index as usize];
                            row.push((pallet_color.0, pallet_color.1, pallet_color.2));
                            current_bit += png.ihdr.bit_depth;

                            x += 1;
                            if (x >= png.ihdr.width) {
                                break 'scanline;
                            }
                        }
                    }
                    colors.push(row);
                }
            }
            _ => {}
        }
    }

    println!("{:#?}", png);
    println!("{:#?}", colors);

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_pngs() {
        let result = read_png("./assets/test_image2.png").unwrap();
        println!("{:#?}", result);
    }
}
