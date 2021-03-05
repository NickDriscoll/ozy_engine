use std::mem;
use std::fs::File;
use std::io::{Error, Read};
use std::string::String;
use crate::structs::*;

pub struct OzyMesh {
	pub vertex_array: VertexArray,
	pub names: Vec<String>,
	pub texture_names: Vec<String>,
	pub geo_boundaries: Vec<u16>,			//[0, a, b, c, ..., indices.length - 1]
	pub origins: Vec<glm::TVec4<f32>>
}

impl OzyMesh {
    pub fn load(path: &str) -> Option<Self> {
        //Open the file
        let mut model_file = match File::open(path) {
            Ok(file) => { file }
            Err(e) => {
                println!("Unable to open \"{}\": {}", path, e);
                return None;
            }
        };
    
        //Read how many meshes are in the file
        let mesh_count = match read_u32(&mut model_file) {
            Ok(count) => { count as usize }
            Err(_) => { return None; }
        };
    
        //Read the geo boundaries
        let geo_boundaries = match read_u16_data(&mut model_file, 1 + mesh_count) {
            Ok(v) => { v }
            Err(_) => { return None; }
        };
    
        //Read the individual mesh names
        let names = match read_pascal_strings(&mut model_file, mesh_count) {
            Ok(v) => { v }
            Err(_) => { return None; }
        };
    
        //Read the material names
        let texture_names = match read_pascal_strings(&mut model_file, mesh_count) {
            Ok(v) => { v }
            Err(_) => { return None; }
        };
    
        //Read the individual mesh origins
        let origins = {
            const VECTOR_COMPONENTS: usize = 3;			//Each origin is three floats (x, y, z) and I didn't like having a magic number
            let mut bytes = vec![0; mesh_count as usize * VECTOR_COMPONENTS * mem::size_of::<f32>()];
            if let Err(e) = model_file.read_exact(bytes.as_mut_slice()) {
                println!("Error reading vertex data from file: {}", e);
                return None;
            }
    
            let mut v = Vec::with_capacity(VECTOR_COMPONENTS * mesh_count);
            let origin_size = mem::size_of::<f32>() * VECTOR_COMPONENTS;
            for i in 0..mesh_count {
                let mut components = [0.0; VECTOR_COMPONENTS];
                for j in 0..VECTOR_COMPONENTS {
                    let component = f32::from_le_bytes([bytes[i * origin_size + j * mem::size_of::<f32>()],
                                                    bytes[i * origin_size + j * mem::size_of::<f32>() + 1],
                                                    bytes[i * origin_size + j * mem::size_of::<f32>() + 2],
                                                    bytes[i * origin_size + j * mem::size_of::<f32>() + 3]
                    ]);
                    components[j] = component;
                }
                
                v.push(glm::vec4(components[0], components[1], components[2], 1.0));
            }
            v
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
            names,
            texture_names,
            geo_boundaries,
            origins
        })
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