use std::mem;
use std::fs::File;
use std::io::{Error, Read};
use std::string::String;
use crate::structs::*;

pub struct OzyMesh {
	pub vertex_array: VertexArray,
	pub texture_name: String,
    pub colors: Vec<f32>
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
            //Read the colors into a Vec
            colors = match read_f32_data(&mut model_file, color_count * 4) {
                Ok(data_block) => { data_block }
                Err(e) => {
                    println!("Error reading color data: {}", e);
                    return None;
                }
            };
        }
    
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
            colors
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

pub fn read_u32(file: &mut File) -> Result<u32, Error> {
	let mut buffer = [0; 4];
	match file.read_exact(&mut buffer) {
		Ok(_) => { Ok(u32::from_le_bytes(buffer)) }
		Err(e) => { Err(e) }
	}
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