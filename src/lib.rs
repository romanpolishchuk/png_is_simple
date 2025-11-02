#![allow(warnings)]
use std::fs::read;

#[derive(Debug)]
struct PNG {
    signature: [u8; 8],
    ihdr: IHDR,
    idat: Vec<IDAT>,
    optional_chunks: Vec<OptionalChunk>,
}

#[derive(Debug)]
enum OptionalChunk {}

#[derive(Debug)]
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
}

#[derive(Debug)]
struct IDAT_RGBA {
    pixels: Vec<(u8, u8, u8, u8)>,
}

fn read_chunks(bytes: &[u8]) -> PNG {
    let signature: [u8; 8] = bytes[0..8].try_into().unwrap();
    let mut ihdr: Option<IHDR> = None;
    let mut idat: Vec<IDAT> = Vec::<IDAT>::new();
    let mut optional_chunks = Vec::<OptionalChunk>::new();

    let mut index = 8;
    while index < bytes.len() {
        let chunk_length = i32::from_be_bytes(bytes[index..index + 4].try_into().unwrap()) as usize;
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
                    bit_depth: u8::from_be_bytes(chunk_data[8..9].try_into().unwrap()),
                    color_type: u8::from_be_bytes(chunk_data[9..10].try_into().unwrap()),
                    compression_method: u8::from_be_bytes(chunk_data[10..11].try_into().unwrap()),
                    filter_method: u8::from_be_bytes(chunk_data[11..12].try_into().unwrap()),
                    interlace_method: u8::from_be_bytes(chunk_data[12..13].try_into().unwrap()),
                });
            }
            "IDAT" => {
                let ihdr = ihdr.expect("IHDR is not present!");

                if ihdr.color_type != 6 {
                    panic!("Unsupported color type: {}", ihdr.color_type);
                }
                if ihdr.bit_depth != 8 {
                    panic!("Unsupported bit depth: {}", ihdr.bit_depth);
                }

                if ihdr.filter_method != 0 {
                    panic!("Unsupported filter method: {}", ihdr.filter_method);
                }
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
        optional_chunks: optional_chunks,
    }
}

pub fn read_png(path: &str) -> Result<Vec<Vec<(u8, u8, u8)>>, &str> {
    let file_bytes = read(path).unwrap();

    let png = read_chunks(&file_bytes);
    println!("{:#?}", png);

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_pngs() {
        let result = read_png("./assets/test_image.png").unwrap();
        println!("{:#?}", result);
    }
}
