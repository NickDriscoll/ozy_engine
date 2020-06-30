use gl::types::*;
use std::mem;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::string::String;
use wavefront_obj::{mtl, obj};
use crate::structs::*;

pub fn flatten_glm(mat: &glm::TMat4<f32>) -> [f32; 16] {
	let slice = glm::value_ptr(mat);

	let mut result = [0.0; 16];
	for i in 0..16 {
		result[i] = slice[i];
	}
	result
}

pub fn uniform_scale(scale: f32) -> glm::TMat4<f32> {
	glm::scaling(&glm::vec3(scale, scale, scale))
}

pub fn load_wavefront_obj(path: &str) -> Option<MeshData> {
	//Gracefully exit if this is not an obj
	if path.split_at(path.len() - 3).1 != "obj" {
		println!("{} is not an obj file.", path);
		return None;
	}

	//Load .obj file's contents as a string
	let obj_contents = match fs::read_to_string(path) {
		Ok(s) => { s }
		Err(e) => {
			println!("Error parsing .obj as string: {}", e);
			return None;
		}
	};

	//Load .mtl file's contents as a string
	let mtl_contents = match fs::read_to_string(format!("{}.mtl", path.split_at(path.len() - 4).0)) {
		Ok(s) => { Some(s) }
		Err(e) => {
			println!("Error parsing .mtl as string: {}", e);
			return None;
		}
	};
	
	//Parse the Objects from the file
	let obj_set = match obj::parse(obj_contents) {
		Ok(m) => { m }
		Err(e) => {
			println!("Error parsing objs: {:?}", e);
			return None;
		}
	};

	//Parse the Materials from the file
	let mtl_set = match mtl_contents {
		Some(contents) => {
			match mtl::parse(contents) {
				Ok(m) => { Some(m) }
				Err(e) => {
					println!("Error parsing materials: {:?}", e);
					return None;
				}
			}
		}
		None => return None
	};

	//Transform the object into something the engine can actually use
	const BUFFER_SIZE: usize = 500;
	let mut index_map = HashMap::new();
	let mut vertices = Vec::with_capacity(BUFFER_SIZE);
	let mut indices = Vec::with_capacity(BUFFER_SIZE);
	let mut geometry_boundaries = Vec::with_capacity(BUFFER_SIZE);
	let mut names = Vec::new();
	let mut materials_in_order = Vec::with_capacity(BUFFER_SIZE);
	let mut current_index = 0u16;
	for object in obj_set.objects {
		names.push(object.name);
		
		for geo in &object.geometry {
			geometry_boundaries.push(indices.len() as GLsizei);

			//Copy the current material into materials_in_order
			match &mtl_set {
				Some(set) => {
					match &geo.material_name {
						Some(name) => {
							for material in &set.materials {
								if *name == material.name {
									materials_in_order.push(material.clone());
									break;
								}
							}
						}
						None => {
							println!("{} doesn't have a material assigned to every object.", path);
							return None;
						}
					}
				}
				None => {
					println!("{} has not mtlset.", path);
					return None;
				}
			}

			for shape in &geo.shapes {
				match shape.primitive {
					obj::Primitive::Triangle(a, b, c) => {
						let verts = [a, b, c];
						for v in &verts {
							let pair = (v.0, v.2, v.1);
							match index_map.get(&pair) {
								Some(i) => {
									//This vertex has already been accounted for, so we can just push the index into indices
									indices.push(*i);
								}
								None => {
									//This vertex is not accounted for, and so now we must add its data to vertices

									//We add the position data to vertices
									vertices.push(object.vertices[pair.0].x as f32);
									vertices.push(object.vertices[pair.0].y as f32);
									vertices.push(object.vertices[pair.0].z as f32);

									//Push the normal vector data if there is any
									match pair.1 {
										Some(i) => {
											let coords = [object.normals[i].x as f32, object.normals[i].y as f32, object.normals[i].z as f32];
											for c in &coords {
												vertices.push(*c);
											}
										}
										None => {
											for _ in 0..3 {
												vertices.push(0.0);
											}
										}
									}

									//Push the texture coordinate data if there is any
									match pair.2 {
										Some(i) => {
											vertices.push(object.tex_vertices[i].u as f32);
											vertices.push(object.tex_vertices[i].v as f32);
										}
										None => {
											vertices.push(0.0);
											vertices.push(0.0);
										}
									}

									//Then we declare that this vertex will appear in vertices at current_index
									index_map.insert(pair, current_index);

									//Add the index to indices
									indices.push(current_index);
									current_index += 1;
								}
							}
						}
					}
					_ => {
						println!("load_wavefront_obj(): Only triangle meshes are supported.");
						return None;
					}
				}
			}
		}
	}
	geometry_boundaries.push(indices.len() as GLsizei);

	let vertex_array = VertexArray {
		vertices,
		indices,
		attribute_offsets: vec![3, 3, 2]
	};

	Some(MeshData {
		vertex_array,
		names,
		geo_boundaries: geometry_boundaries,
		materials: materials_in_order
	})
}

fn read_u32(file: &mut File, error_message: &str) -> Option<u32> {
	let mut buffer = [0; 4];
	match file.read_exact(&mut buffer) {
		Ok(_) => { Some(u32::from_le_bytes(buffer)) }
		Err(e) => {
			println!("{}: {}", error_message, e);
			None
		}
	}
}

fn read_u16_data(file: &mut File, count: usize) -> Option<Vec<u16>> {
	let mut bytes = vec![0; count * mem::size_of::<u16>()];
	if let Err(e) = file.read_exact(bytes.as_mut_slice()) {
		println!("Error reading data from file: {}", e);
		return None;
	}

	let mut v = Vec::with_capacity(count);
	for i in (0..bytes.len()).step_by(mem::size_of::<u16>()) {
		let b = [bytes[i], bytes[i + 1]];
		v.push(u16::from_le_bytes(b));
	}
	Some(v)
}

fn read_pascal_strings(file: &mut File, count: usize) -> Option<Vec<String>> {	
	let mut int_buffer = [0; 4];
	let mut names = Vec::with_capacity(count as usize);
	for _ in 0..count {
		//Read the name's length
		if let Err(e) = file.read_exact(&mut int_buffer) {
			println!("Error read string length from file: {}", e);
			return None;
		}

		//Read the contents of the string into a buffer
		let mut utf8_buffer = vec![0; u32::from_le_bytes(int_buffer) as usize];
		if let Err(e) = file.read_exact(&mut utf8_buffer) {
			println!("Error reading string contents from file: {}", e);
			return None;
		}

		//Turn buffer of utf-8 characters into String
		match String::from_utf8(utf8_buffer) {
			Ok(name) => { names.push(name); }
			Err(e) => {
				println!("Couldn't decode string: {}", e);
				return None;
			}
		}
	}
	Some(names)
}

//Loads a file of the proprietary format OzyMesh
pub fn load_ozymesh(path: &str) -> Option<OzyMesh> {
	let mut int_buffer = [0x0; 4];				//Buffer for extracting the u32s from the file that represent the lengths of the data sections

	//Open the file
	let mut model_file = match File::open(path) {
		Ok(file) => { file }
		Err(e) => {
			println!("Unable to open ozymesh file: {}", e);
			return None;
		}
	};

	//Read how many meshes are in the file
	let mesh_count = match model_file.read_exact(&mut int_buffer) {
		Ok(_) => { u32::from_le_bytes(int_buffer) }
		Err(e) => {
			println!("Error reading mesh_count: {}", e);
			return None;
		}
	};

	//Read the geo boundaries
	let geo_boundaries = match read_u16_data(&mut model_file, 1 + mesh_count as usize) {
		Some(v) => { v }
		None => { return None; }
	};

	//Read the node ids
	let node_ids = match read_u16_data(&mut model_file, mesh_count as usize) {
		Some(v) => { v }
		None => { return None; }
	};

	//Read the node parent ids
	let node_parent_ids = match read_u16_data(&mut model_file, mesh_count as usize) {
		Some(v) => { v }
		None => { return None; }
	};

	//Read all of the names
	let names = match read_pascal_strings(&mut model_file, mesh_count as usize) {
		Some(v) => { v }
		None => { return None; }
	};

	//Read the material names
	let texture_names = match read_pascal_strings(&mut model_file, mesh_count as usize) {
		Some(v) => { v }
		None => { return None; }
	};

	let vertices_size = match read_u32(&mut model_file, "Error reading vertex_count") {
		Some(n) => { n }
		None => { return None; }
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
	
	let index_count = match read_u32(&mut model_file, "Error reading index_count") {
		Some(n) => { n / mem::size_of::<u16>() as u32 }
		None => { return None; }
	};

	
	let indices = match read_u16_data(&mut model_file, index_count as usize) {
		Some(n) => { n }
		None => { return None; }
	};

	let vertex_array = VertexArray {
		vertices,
		indices,
		attribute_offsets: vec![3, 2]
	};

	Some(OzyMesh {
		vertex_array,
		names,
		texture_names,
		geo_boundaries,
		node_ids,
		parent_ids: node_parent_ids
	})
}