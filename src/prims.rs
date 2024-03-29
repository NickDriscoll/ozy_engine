
use std::{collections::HashMap};

use crate::{glutil, structs::UninterleavedVertexArrays};

pub fn sphere_index_count(segments: usize, rings: usize) -> usize {	
	6 * (segments * (rings - 2) + segments)
}

pub fn sphere_vertex_buffer(radius: f32, segments: usize, rings: usize) -> Vec<f32> {
	let attrib_offsets = [3, 3];
	let attrib_size = {
		let mut s = 0;
		for att in attrib_offsets.iter() {
			s += *att as usize;
		}
		s
	};
	let mut verts = vec![0.0; attrib_size * (segments * (rings - 1) + 2)];

	//Compute the vertices of the sphere
	verts[0] = 0.0;
	verts[1] = -radius;
	verts[2] = 0.0;
	verts[3] = 0.0;
	verts[4] = -radius;
	verts[5] = 0.0;
	verts[6] = 0.0;
	verts[7] = radius;
	verts[8] = 0.0;
	verts[9] = 0.0;
	verts[10] = radius;
	verts[11] = 0.0;

	for i in 0..(rings - 1) {
		let r = i as f32 + 1.0;
		let phi = glm::pi::<f32>() * r / rings as f32 - glm::half_pi::<f32>();
		for j in 0..segments {
			let s = j as f32;
			let theta = glm::two_pi::<f32>() * s / segments as f32;

			//Compute base index of this vertex
			let vert = attrib_size * (i * segments + j + 2);

			//Write position data
			verts[vert] = radius * f32::cos(theta) * f32::cos(phi);
			verts[vert + 2] = radius * f32::sin(theta) * f32::cos(phi);
			verts[vert + 1] = radius * f32::sin(phi);

			//Write normal data
			verts[vert + 3] = verts[vert];
			verts[vert + 4] = verts[vert + 2];
			verts[vert + 5] = verts[vert + 1];
		}
	}

	let verts = verts.iter().map(|v|{ *v * -1.0 }).collect();

	verts
}

pub fn sphere_index_array(segments: usize, rings: usize) -> Vec<u16> {
	//Compute sphere index data
	let mut inds = vec![0u16; sphere_index_count(segments, rings)];

	let segs = segments.into();
	for i in 0..(rings as usize - 2) {
		let offset = i * segments;
		for j in 0..segs {
			let ind = 6 * (offset + j as usize);
			
			let offset = offset as u16;
			let j = j as u16;
			let segs = segs as u16;
			inds[ind] = offset + j + 2;
			inds[ind + 1] = offset + j + 1 + 2;
			inds[ind + 2] = offset + j + segs + 2;
			inds[ind + 3] = offset + j + segs + 1 + 2;
			inds[ind + 4] = offset + j + segs + 2;
			inds[ind + 5] = offset + j + 1 + 2;
			
			if j == segs - 1 {
				inds[ind + 1] = offset + 2;
				inds[ind + 5] = offset + 2;
				inds[ind + 3] = offset + segs + 2;
			}
		}
	}

	let base_index = 6 * (segments * (rings - 2));
	for i in 0..segments {
		inds[base_index + i * 3] = 0;
		inds[base_index + i * 3 + 1] = i as u16 + 3;
		inds[base_index + i * 3 + 2] = i as u16 + 2;
		
		if i == segments - 1 {
			inds[base_index + i * 3 + 1] = 2;
		}
	}

	let base_index = 6 * (segments * (rings - 2)) + segments * 3;
	for i in 0..segments {
		inds[base_index + i * 3] = 1;
		inds[base_index + i * 3 + 1] = (rings as u16 - 2) * segs as u16 + i as u16 + 2;
		inds[base_index + i * 3 + 2] = (rings as u16 - 2) * segs as u16 + i as u16 + 3;
		
		if i == segments - 1 {
			inds[base_index + i * 3 + 2] = (rings as u16 - 2) * segs as u16 + 2;
		}
	}

	inds
}

pub fn sphere_vao(radius: f32, segments: usize, rings: usize) -> glutil::VertexArrayNames {
	let attrib_offsets = [3, 3];

	//Compute sphere vertex and index data
	let verts = sphere_vertex_buffer(radius, segments, rings);
	let inds = sphere_index_array(segments, rings);

	unsafe { glutil::create_vertex_array_object(&verts, &inds, &attrib_offsets) }
}

pub fn debug_sphere_vao(radius: f32, segments: usize, rings: usize) -> glutil::VertexArrayNames {
	let attrib_offsets = [3, 3];
	let attrib_size = {
		let mut s = 0;
		for att in attrib_offsets.iter() {
			s += *att as usize;
		}
		s
	};
	let mut verts = vec![0.0; attrib_size * (segments * (rings - 1) + 2)];

	//Compute the vertices of the sphere
	verts[0] = 0.0;
	verts[1] = -radius;
	verts[2] = 0.0;
	verts[3] = 0.0;
	verts[4] = -radius;
	verts[5] = 0.0;
	verts[6] = 0.0;
	verts[7] = radius;
	verts[8] = 0.0;
	verts[9] = 0.0;
	verts[10] = radius;
	verts[11] = 0.0;

	for i in 0..(rings - 1) {
		let r = i as f32 + 1.0;
		let phi = glm::pi::<f32>() * r / rings as f32 - glm::half_pi::<f32>();
		for j in 0..segments {
			let s = j as f32;
			let theta = glm::two_pi::<f32>() * s / segments as f32;

			//Compute base index of this vertex
			let vert = attrib_size * (i * segments + j + 2);

			//Write position data
			verts[vert] = radius * f32::cos(theta) * f32::cos(phi);
			verts[vert + 2] = radius * f32::sin(theta) * f32::cos(phi);
			verts[vert + 1] = radius * f32::sin(phi);

			//Write normal data			
			verts[vert + 3] = radius * f32::cos(theta) * f32::cos(phi);
			verts[vert + 5] = radius * f32::sin(theta) * f32::cos(phi);
			verts[vert + 4] = radius * f32::sin(phi);
		}
	}

	//Compute sphere index data
	let inds = sphere_index_array(segments, rings);

	unsafe { glutil::create_vertex_array_object(&verts, &inds, &attrib_offsets) }
}

/*
  (-1.0, 1.0) plane with right-handed z-up
  Vertex attributes are as follows:
	vec3 position
	vec3 tangent;
	vec3 bitangent;
	vec3 normal;
	vec2 uv;
//
*/
pub fn plane_index_buffer(width: usize, height: usize) -> Vec<u32> {
	let mut indices = vec![0u32; (width - 1) * (height - 1) * 6];
	for i in 0..(height - 1) {
		for j in 0..(width - 1) {
			let current_square = i * (width - 1) + j;
			indices[current_square * 6] =     (current_square + i) as u32;
			indices[current_square * 6 + 1] = (current_square + i + 1) as u32;
			indices[current_square * 6 + 2] = (current_square + width + i) as u32;
			indices[current_square * 6 + 3] = (current_square + i + 1) as u32;
			indices[current_square * 6 + 4] = (current_square + width + i + 1) as u32;
			indices[current_square * 6 + 5] = (current_square + width + i) as u32;
		}
	}
	indices
}

pub fn plane_vertex_buffer(width: usize, height: usize, scale: f32) -> Vec<f32> {
	let floats_per_vertex = 14;
	let mut vertex_buffer = vec![0.0; width * height * floats_per_vertex];

	for j in 0..height {
		let ypos = j as f32 * 2.0 / (height - 1) as f32 - 1.0;
		let yuv = j as f32 / (height - 1) as f32;
		let row_index = j * width * floats_per_vertex;
		for i in 0..width {
			let xpos = i as f32 * 2.0 / (width - 1) as f32 - 1.0;
			let xuv = i as f32 / (width - 1) as f32;

			let vertex_offset = row_index + i * floats_per_vertex;

			vertex_buffer[vertex_offset] =     xpos * scale;
			vertex_buffer[vertex_offset + 1] = ypos * scale;
			vertex_buffer[vertex_offset + 2] = 0.0;

			vertex_buffer[vertex_offset + 3] = 1.0;
			vertex_buffer[vertex_offset + 4] = 0.0;
			vertex_buffer[vertex_offset + 5] = 0.0;

			vertex_buffer[vertex_offset + 6] = 0.0;
			vertex_buffer[vertex_offset + 7] = 1.0;
			vertex_buffer[vertex_offset + 8] = 0.0;

			vertex_buffer[vertex_offset + 9] = 0.0;
			vertex_buffer[vertex_offset + 10] = 0.0;
			vertex_buffer[vertex_offset + 11] = 1.0;
			
			vertex_buffer[vertex_offset + 12] = xuv * scale;
			vertex_buffer[vertex_offset + 13] = yuv * scale;

		}
	}

	vertex_buffer
}

pub fn perturbed_plane_vertex_buffer<HeightMapper: Fn(f64, f64) -> f64>(width: usize, height: usize, scale: f32, generator: HeightMapper) -> UninterleavedVertexArrays {
	//let floats_per_vertex = 15;
	let mut vertex_positions = vec![0.0; width * height * 4];
	let mut vertex_normals = vec![0.0; width * height * 4];
	let mut vertex_tangents = vec![0.0; width * height * 4];
	let mut vertex_uvs = vec![0.0; width * height * 2];

	let mut face_normals = vec![glm::zero(); 2 * (width - 1) * (height - 1)];
	let mut face_tangents = vec![glm::zero(); 2 * (width - 1) * (height - 1)];
	let mut face_bitangents = vec![glm::zero(); 2 * (width - 1) * (height - 1)];

	//Initial pass to fill out positions and uv-coordinates
	for j in 0..height {
		let ypos = j as f32 * 2.0 / (height - 1) as f32 - 1.0;
		let yuv = j as f32 / (height - 1) as f32;
		let row_index = j * width;
		for i in 0..width {
			let xpos = i as f32 * 2.0 / (width - 1) as f32 - 1.0;
			let xuv = i as f32 / (width - 1) as f32;

			let vertex_offset = row_index + i;
			let pos_offset = 4 * vertex_offset;
			let uv_offset = 2 * vertex_offset;

			let x = xpos * scale;
			let y = ypos * scale;
			let z = generator(x as f64, y as f64) as f32;
			vertex_positions[pos_offset] =     x * scale;
			vertex_positions[pos_offset + 1] = y * scale;
			vertex_positions[pos_offset + 2] = z * scale;
			vertex_positions[pos_offset + 3] = 1.0;
			
			vertex_uvs[uv_offset] = xuv * scale;
			vertex_uvs[uv_offset + 1] = yuv * scale;
		}
	}
	
	//Build map of vertex indices to lists of faces
	let index_buffer = plane_index_buffer(width, height);
	let mut vertex_face_map: HashMap<u32, Vec<u32>> = HashMap::with_capacity(index_buffer.len() / 3);

	//Iterating over each two-triangle square to get the TBN vectors
	//Tangent and Bitangent formula lifted from here https://www.cs.upc.edu/~virtual/G/1.%20Teoria/06.%20Textures/Tangent%20Space%20Calculation.pdf
	for j in 0..(height - 1) {
		for i in 0..(width - 1) {			
			//Compute index of this square
			let square_index = j * (width - 1) + i;

			//Get the four indices of this square's vertices
			let i0 = j * width + i;
			let i1 = j * width + i + 1;
			let i2 = j * width + i + width;
			let i3 = j * width + i + width + 1;

			let tri_id = 2 * square_index as u32;
			match vertex_face_map.get_mut(&(i0 as u32)) {
				Some(list) => { list.push(tri_id); }
				None => { vertex_face_map.insert(i0 as u32, vec![tri_id]); }
			}
			match vertex_face_map.get_mut(&(i1 as u32)) {
				Some(list) => { list.push(tri_id); }
				None => { vertex_face_map.insert(i1 as u32, vec![tri_id]); }
			}
			match vertex_face_map.get_mut(&(i2 as u32)) {
				Some(list) => { list.push(tri_id); }
				None => { vertex_face_map.insert(i2 as u32, vec![tri_id]); }
			}

			let tri_id = 2 * square_index as u32 + 1;
			match vertex_face_map.get_mut(&(i1 as u32)) {
				Some(list) => { list.push(tri_id); }
				None => { vertex_face_map.insert(i1 as u32, vec![tri_id]); }
			}
			match vertex_face_map.get_mut(&(i2 as u32)) {
				Some(list) => { list.push(tri_id); }
				None => { vertex_face_map.insert(i2 as u32, vec![tri_id]); }
			}
			match vertex_face_map.get_mut(&(i3 as u32)) {
				Some(list) => { list.push(tri_id); }
				None => { vertex_face_map.insert(i3 as u32, vec![tri_id]); }
			}

			//First tri
			let p0 = glm::vec3(vertex_positions[4 * i0], vertex_positions[4 * i0 + 1], vertex_positions[4 * i0 + 2]);
			let p1 = glm::vec3(vertex_positions[4 * i1], vertex_positions[4 * i1 + 1], vertex_positions[4 * i1 + 2]);
			let p2 = glm::vec3(vertex_positions[4 * i2], vertex_positions[4 * i2 + 1], vertex_positions[4 * i2 + 2]);

			//Compute normal
			let e1 = p1 - p0;
			let e2 = p2 - p0;
			let face_normal = glm::normalize(&glm::cross(&e1, &e2));
			face_normals[square_index * 2] = face_normal;

			//Now computing tangent and bitangent
			let (u0, v0) = (vertex_uvs[2 * i0], vertex_uvs[2 * i0 + 1]);
			let (u1, v1) = (vertex_uvs[2 * i1], vertex_uvs[2 * i1 + 1]);
			let (u2, v2) = (vertex_uvs[2 * i2], vertex_uvs[2 * i2 + 1]);
			let q1 = p1 - p0;
			let q2 = p2 - p0;
			let (s1, t1) = (u1 - u0, v1 - v0);
			let (s2, t2) = (u2 - u0, v2 - v0);
			let raw_tanbitan = 1.0 / (s1 * t2 - s2 * t1) * glm::mat2(t2, -t1, -s2, s1) * glm::mat2x3(q1.x, q1.y, q1.z, q2.x, q2.y, q2.z);
			let face_tangent = glm::normalize(&glm::vec3(raw_tanbitan[0], raw_tanbitan[2], raw_tanbitan[4]));
			let face_bitangent = glm::normalize(&glm::vec3(raw_tanbitan[1], raw_tanbitan[3], raw_tanbitan[5]));

			face_tangents[square_index * 2] = face_tangent;
			face_bitangents[square_index * 2] = face_bitangent;

			//Second tri
			let p0 = glm::vec3(vertex_positions[4 * i1], vertex_positions[4 * i1 + 1], vertex_positions[4 * i1 + 2]);
			let p1 = glm::vec3(vertex_positions[4 * i2], vertex_positions[4 * i2 + 1], vertex_positions[4 * i2 + 2]);
			let p2 = glm::vec3(vertex_positions[4 * i3], vertex_positions[4 * i3 + 1], vertex_positions[4 * i3 + 2]);

			//Compute normal
			let e1 = p1 - p0;
			let e2 = p2 - p0;
			let face_normal = glm::normalize(&glm::cross(&e2, &e1));
			face_normals[square_index * 2 + 1] = face_normal;

			//Now computing tangent and bitangent
			let (u0, v0) = (vertex_positions[2 * i1], vertex_positions[2 * i1 + 1]);
			let (u1, v1) = (vertex_positions[2 * i2], vertex_positions[2 * i2 + 1]);
			let (u2, v2) = (vertex_positions[2 * i3], vertex_positions[2 * i3 + 1]);
			let q1 = p1 - p0;
			let q2 = p2 - p0;
			let (s1, t1) = (u1 - u0, v1 - v0);
			let (s2, t2) = (u2 - u0, v2 - v0);
			let raw_tanbitan = 1.0 / (s1 * t2 - s2 * t1) * glm::mat2(t2, -t1, -s2, s1) * glm::mat2x3(q1.x, q1.y, q1.z, q2.x, q2.y, q2.z);
			let face_tangent = glm::normalize(&glm::vec3(raw_tanbitan[0], raw_tanbitan[2], raw_tanbitan[4]));
			let face_bitangent = glm::normalize(&glm::vec3(raw_tanbitan[1], raw_tanbitan[3], raw_tanbitan[5]));

			face_tangents[square_index * 2 + 1] = face_tangent;
			face_bitangents[square_index * 2 + 1] = face_bitangent;
		}
	}

	//Averaging per-face data into vertex data
	//for i in (0..vertex_buffer.len()).step_by(floats_per_vertex) {
	for i in 0..(vertex_positions.len() / 4) {
		let vert_id = i as u32;
		let vert_faces = vertex_face_map.get(&vert_id).unwrap();

		let mut averaged_tangent: glm::TVec3<f32> = glm::zero();
		for &face_id in vert_faces {
			averaged_tangent += face_tangents[face_id as usize];
		}
		averaged_tangent = glm::normalize(&averaged_tangent);

		let mut averaged_normal: glm::TVec3<f32> = glm::zero();
		for &face_id in vert_faces {
			averaged_normal += face_normals[face_id as usize];
		}
		averaged_normal = glm::normalize(&averaged_normal);

		vertex_tangents[4 * i] = averaged_tangent.x;
		vertex_tangents[4 * i + 1] = averaged_tangent.y;
		vertex_tangents[4 * i + 2] = averaged_tangent.z;
		vertex_tangents[4 * i + 3] = 1.0;

		vertex_normals[4 * i] = averaged_normal.x;
		vertex_normals[4 * i + 1] = averaged_normal.y;
		vertex_normals[4 * i + 2] = averaged_normal.z;
		vertex_normals[4 * i + 3] = 0.0;
	}

	UninterleavedVertexArrays {
		positions: vertex_positions,
		normals: vertex_normals,
		tangents: vertex_tangents,
		uvs: vertex_uvs
	}
}

pub fn plane_vao(vertices_width: usize) -> glutil::VertexArrayNames {
	if vertices_width < 2 {
		panic!("vertices_width must be greater than 2");
	}

	let floats_per_vertex = 14;
	let mut vertex_buffer = vec![0.0; vertices_width * vertices_width * floats_per_vertex];
	let mut indices = vec![0u16; (vertices_width-1)*(vertices_width-1) * 2 * 3];	
	let attribute_offsets = [3, 3, 3, 3, 2];

	//Filling out the vertex buffer
	//Right-handed z-up, looking down at the x-y plane, this goes left-right, bottom-up
	for i in 0..vertices_width {
		let ypos = i as f32 * 2.0 / (vertices_width - 1) as f32 - 1.0;
		let yuv = i as f32 / (vertices_width - 1) as f32;
		let row_index = i * vertices_width * floats_per_vertex;

		for j in 0..vertices_width {
			let xpos = j as f32 * 2.0 / (vertices_width - 1) as f32 - 1.0;
			let xuv = j as f32 / (vertices_width - 1) as f32;
			let vertex_offset = row_index + j * floats_per_vertex;

			vertex_buffer[vertex_offset] =     xpos;
			vertex_buffer[vertex_offset + 1] = ypos;
			vertex_buffer[vertex_offset + 2] = 0.0;

			vertex_buffer[vertex_offset + 3] = 1.0;
			vertex_buffer[vertex_offset + 4] = 0.0;
			vertex_buffer[vertex_offset + 5] = 0.0;

			vertex_buffer[vertex_offset + 6] = 0.0;
			vertex_buffer[vertex_offset + 7] = 1.0;
			vertex_buffer[vertex_offset + 8] = 0.0;

			vertex_buffer[vertex_offset + 9] = 0.0;
			vertex_buffer[vertex_offset + 10] = 0.0;
			vertex_buffer[vertex_offset + 11] = 1.0;
			
			vertex_buffer[vertex_offset + 12] = xuv;
			vertex_buffer[vertex_offset + 13] = yuv;
		}
	}

	//Filling out the index buffer
	for i in 0..(vertices_width - 1) {
		for j in 0..(vertices_width - 1) {
			let current_square = i * (vertices_width - 1) + j;

			indices[current_square * 6] =     (current_square + i) as u16;
			indices[current_square * 6 + 1] = (current_square + i + 1) as u16;
			indices[current_square * 6 + 2] = (current_square + vertices_width + i) as u16;
			indices[current_square * 6 + 3] = (current_square + i + 1) as u16;
			indices[current_square * 6 + 4] = (current_square + vertices_width + i + 1) as u16;
			indices[current_square * 6 + 5] = (current_square + vertices_width + i) as u16;
		}
	}

	unsafe { glutil::create_vertex_array_object(&vertex_buffer, &indices, &attribute_offsets) }
}

pub fn skybox_cube_vertex_buffer() -> Vec<f32> {
	vec![
		-1.0, -1.0, -1.0,
		1.0, -1.0, -1.0,
		-1.0, 1.0, -1.0,
		1.0, 1.0, -1.0,
		-1.0, -1.0, 1.0,
		-1.0, 1.0, 1.0,
		1.0, -1.0, 1.0,
		1.0, 1.0, 1.0
	]
}

pub fn skybox_cube_index_buffer() -> Vec<u32> {
	vec![
		//Front
		0, 1, 2,
		3, 2, 1,
        
        //Left
		0, 2, 4,
		2, 5, 4,

		//Right
		3, 1, 6,
		7, 3, 6,

		//Back
		5, 7, 4,
		7, 6, 4,

		//Bottom
	    4, 1, 0,
    	4, 6, 1,
        
        //Top
		7, 5, 2,
		7, 2, 3
	]
}

pub fn skybox_cube_vao() -> glutil::VertexArrayNames {
	const VERTICES: [f32; 24] = [
		-1.0, -1.0, -1.0,
		1.0, -1.0, -1.0,
		-1.0, 1.0, -1.0,
		1.0, 1.0, -1.0,
		-1.0, -1.0, 1.0,
		-1.0, 1.0, 1.0,
		1.0, -1.0, 1.0,
		1.0, 1.0, 1.0
	];
	const INDICES: [u16; 36] = [
		//Front
		0, 1, 2,
		3, 2, 1,
        
        //Left
		0, 2, 4,
		2, 5, 4,

		//Right
		3, 1, 6,
		7, 3, 6,

		//Back
		5, 7, 4,
		7, 6, 4,

		//Bottom
	    4, 1, 0,
    	4, 6, 1,
        
        //Top
		7, 5, 2,
		7, 2, 3
	];
	
	unsafe { glutil::create_vertex_array_object(&VERTICES, &INDICES, &[3]) }
}
