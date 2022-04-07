use std::mem;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::string::String;
use crate::structs::*;

/*
struct DDS_PIXELFORMAT {
  DWORD dwSize;
  DWORD dwFlags;
  DWORD dwFourCC;
  DWORD dwRGBBitCount;
  DWORD dwRBitMask;
  DWORD dwGBitMask;
  DWORD dwBBitMask;
  DWORD dwABitMask;
};
*/
#[derive(Default)]
pub struct DDS_PixelFormat {
    size: u32,
    flags: u32,
    four_cc: u32,
    rgb_bitcount: u32,
    r_bitmask: u32,
    g_bitmask: u32,
    b_bitmask: u32,
    a_bitmask: u32,
}

/*
typedef struct {
     DWORD           dwSize;
     DWORD           dwFlags;
     DWORD           dwHeight;
     DWORD           dwWidth;
     DWORD           dwPitchOrLinearSize;
     DWORD           dwDepth;
     DWORD           dwMipMapCount;
     DWORD           dwReserved1[11];
     DDS_PIXELFORMAT ddspf;
     DWORD           dwCaps;
     DWORD           dwCaps2;
     DWORD           dwCaps3;
     DWORD           dwCaps4;
     DWORD           dwReserved2;
} DDS_HEADER;
   */
#[derive(Default)]
pub struct DDSHeader {
    size: u32,
    flags: u32,
    height: u32,
    width: u32,
    pitch_or_linear_size: u32,
    depth: u32,
    mipmap_count: u32,
    reserved_1: [u32; 11],               //We love fields like this, don't we Microsoft?
    spf: DDS_PixelFormat,
    caps: u32,
    caps2: u32,
    caps3: u32,
    caps4: u32,
    reserved2: u32
}

//This function takes a dds file and gets the width and height of the image
pub fn read_dds_dimensions(dds_file: &mut File) -> (u32, u32) {
    const BC7_HEADER_SIZE: usize = 148;
    let mut header_buffer = vec![0u8; BC7_HEADER_SIZE];

    dds_file.read_exact(&mut header_buffer).unwrap();
    
    const DDSD_HEIGHT: u32 = 0x2;
    const DDSD_WIDTH: u32 = 0x4;
    let flags = u32::from_le_bytes([header_buffer[4], header_buffer[5], header_buffer[6], header_buffer[7]]);

    let height_bytes = [header_buffer[8], header_buffer[9], header_buffer[10], header_buffer[11]];
    let width_bytes = [header_buffer[12], header_buffer[13], header_buffer[14], header_buffer[15]];
    
    let width = u32::from_le_bytes(width_bytes);
    let height;
    if flags & DDSD_HEIGHT == 0 {
        height = width;
    } else {
        height = u32::from_le_bytes(height_bytes);
    }

    println!("Every u32 in the header in order:");
    for i in 0..BC7_HEADER_SIZE/4 {
        let value = u32::from_le_bytes([header_buffer[4 * i], header_buffer[4 * i + 1], header_buffer[4 * i + 2], header_buffer[4 * i + 3]]);
        println!("{}", value);
    }

    (width, height)
}

#[derive(Debug)]
pub struct OzyMesh {
	pub vertex_array: VertexArray,
	pub texture_name: String,
    pub uv_velocity: [f32; 2],
    pub colors: Vec<f32>,
    pub is_transparent: bool
}

impl OzyMesh {
    pub fn load(path: &str) -> Option<Self> {
        let mut texture_name = String::new();
        let mut colors = vec![];

        //Open the file
        let mut model_file = match File::open(path) {
            Ok(file) => { file }
            Err(e) => {
                println!("Unable to open \"{}\": {}", path, e);
                return None;
            }
        };

        //Check how many solid colors there are
        //If 0, it means this model uses textures
        let color_count = match read_u8(&mut model_file) {
            Ok(count) => { count as usize }
            Err(e) => {
                println!("{}", e);
                return None;
            }
        };

        //Branching on whether or not the model is textured or uses solid colors
        if color_count == 0 {
            //Read the material name
            texture_name = match read_pascal_strings(&mut model_file, 1) {
                Ok(v) => { v[0].clone() }
                Err(_) => { return None; }
            };
        } else {
            //Read the color f32s into a Vec
            colors = match read_f32_data(&mut model_file, color_count * 4) {
                Ok(data_block) => { data_block }
                Err(e) => {
                    println!("Error reading color data: {}", e);
                    return None;
                }
            };
        }

        //Transparency flag
        let is_transparent = match read_u8(&mut model_file) {
            Ok(flag) => { flag != 0 }
            Err(e) => {
                println!("Error reading transparency flag: {}", e);
                return None;
            }
        };

        //The uv_velocity
        let uv_velocity = match read_f32_data(&mut model_file, 2) {
            Ok(data) => { [data[0], data[1]] }
            Err(e) => { 
                println!("Error reading uv_velocity: {}", e);
                return None;
            }
        };
    
        //The length of the vertex data section of the file, in bytes
        let vertices_size = match read_u32(&mut model_file) {
            Ok(n) => { n }
            Err(_) => { return None; }
        };
    
        let vertices = {
            let mut bytes = vec![0; vertices_size as usize];
            if let Err(e) = model_file.read_exact(bytes.as_mut_slice()) {
                println!("Error reading vertex data from file: {}", e);
                return None;
            }
    
            let mut v = Vec::with_capacity(vertices_size as usize / mem::size_of::<f32>());
            for i in (0..bytes.len()).step_by(mem::size_of::<f32>()) {
                let b = [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]];
                v.push(f32::from_le_bytes(b));
            }
            v
        };
        
        let index_count = match read_u32(&mut model_file) {
            Ok(n) => { (n / mem::size_of::<u16>() as u32) as usize }
            Err(_) => { return None; }
        };
        
        let indices = match read_u16_data(&mut model_file, index_count) {
            Ok(n) => { n }
            Err(_) => { return None; }
        };
    
        let vertex_array = VertexArray {
            vertices,
            indices,
            attribute_offsets: vec![3, 3, 3, 3, 2]
        };
    
        Some(OzyMesh {
            vertex_array,
            texture_name,
            uv_velocity,
            colors,
            is_transparent
        })
    }
}

pub fn read_u8(file: &mut File) -> Result<u8, Error> {
    let mut n = [0];
    match file.read_exact(&mut n) {
        Ok(_) => { Ok(u8::from_le_bytes(n)) }
        Err(e) => { Err(e) }
    }    
}

pub fn read_u8_data(file: &mut File, count: usize) -> Result<Vec<u8>, Error> {
    let mut bytes = vec![0; count];
    if let Err(e) = file.read_exact(bytes.as_mut_slice()) {
        return Err(e);
	}
	Ok(bytes)
}

pub fn read_u32(file: &mut File) -> Result<u32, Error> {
	let mut buffer = [0; 4];
	match file.read_exact(&mut buffer) {
		Ok(_) => { Ok(u32::from_le_bytes(buffer)) }
		Err(e) => { Err(e) }
	}
}

pub fn read_u32_data(file: &mut File, count: usize) -> Result<Vec<u32>, Error> {
	let mut bytes = vec![0; count * mem::size_of::<u32>()];
	if let Err(e) = file.read_exact(bytes.as_mut_slice()) {
        return Err(e);
	}

	let mut v = Vec::with_capacity(count);
	for i in (0..bytes.len()).step_by(mem::size_of::<u32>()) {
		let b = [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]];
		v.push(u32::from_le_bytes(b));
	}
	Ok(v)
}

pub fn read_u16_data(file: &mut File, count: usize) -> Result<Vec<u16>, Error> {
	let mut bytes = vec![0; count * mem::size_of::<u16>()];
	if let Err(e) = file.read_exact(bytes.as_mut_slice()) {
        return Err(e);
	}

	let mut v = Vec::with_capacity(count);
	for i in (0..bytes.len()).step_by(mem::size_of::<u16>()) {
		let b = [bytes[i], bytes[i + 1]];
		v.push(u16::from_le_bytes(b));
	}
	Ok(v)
}

pub fn read_f32_data(file: &mut File, count: usize) -> Result<Vec<f32>, Error> {
	let mut bytes = vec![0; count * mem::size_of::<f32>()];
	if let Err(e) = file.read_exact(bytes.as_mut_slice()) {
        return Err(e);
	}

	let mut v = Vec::with_capacity(count);
	for i in (0..bytes.len()).step_by(mem::size_of::<f32>()) {
		let b = [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]];
		v.push(f32::from_le_bytes(b));
	}
	Ok(v)    
}

pub fn read_pascal_strings(file: &mut File, count: usize) -> Result<Vec<String>, Error> {	
	let mut int_buffer = [0; 4];
	let mut strings = Vec::with_capacity(count as usize);
	for _ in 0..count {
		//Read the name's length
		if let Err(e) = file.read_exact(&mut int_buffer) {
			return Err(e);
		}

		//Read the contents of the string into a buffer
		let mut utf8_buffer = vec![0; u32::from_le_bytes(int_buffer) as usize];
		if let Err(e) = file.read_exact(&mut utf8_buffer) {
			return Err(e);
		}

		//Turn buffer of utf-8 bytes into String
		match String::from_utf8(utf8_buffer) {
			Ok(name) => { strings.push(name); }
			Err(e) => {
				panic!("read_pascal_strings(): Error when making String from utf8: {}", e);
			}
		}
	}
	Ok(strings)
}

pub fn write_pascal_strings(file: &mut File, strs: &[&str]) {
    for i in 0..strs.len() {
        let s = strs[i];
        let l = s.len() as u32;
        if let Err(e) = file.write(&u32::to_le_bytes(l)) {
            panic!("Couldn't write pascal strings: {}", e);
        }        

        if let Err(e) = file.write(s.as_bytes()) {
            panic!("Couldn't write pascal strings: {}", e);
        }
    }
}