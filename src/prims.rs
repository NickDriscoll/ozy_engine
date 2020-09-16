use gl::types::*;
use crate::glutil;

pub fn sphere_index_count(segments: usize, rings: usize) -> usize {	
	segments * (rings - 2) * 6 + 6 * segments
	//segments * (rings - 2) * 6
}

pub fn sphere_vao(radius: f32, segments: usize, rings: usize) -> GLuint {
	let attrib_offsets = [3, 3];
	let attrib_size = {
		let mut s = 0;
		for att in attrib_offsets.iter() {
			s += *att as usize;
		}
		s
	};
	let mut verts = vec![0.0; attrib_size * (segments * (rings - 1) + 2)];
	let mut inds = vec![0u16; sphere_index_count(segments, rings)];

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

	//Compute sphere index data
	let segs = segments as u16;
	for i in 0..(rings - 2) {
		let offset = i * segments;
		for j in 0..segs {
			let ind = 6 * (offset + j as usize);
			
			inds[ind] = offset as u16 + j + 2;
			inds[ind + 1] = offset as u16 + j + 1 + 2;
			inds[ind + 2] = offset as u16 + j + segs + 2;
			inds[ind + 3] = offset as u16 + j + segs + 1 + 2;
			inds[ind + 4] = offset as u16 + j + segs + 2;
			inds[ind + 5] = offset as u16 + j + 1 + 2;
			
			if j == segs - 1 {
				inds[ind + 1] = offset as u16 + 2;
				inds[ind + 5] = offset as u16 + 2;
				inds[ind + 3] = offset as u16 + segs + 2;
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
		inds[base_index + i * 3 + 1] = (rings as u16 - 2) * segs + i as u16 + 2;
		inds[base_index + i * 3 + 2] = (rings as u16 - 2) * segs + i as u16 + 3;
		
		if i == segments - 1 {
			inds[base_index + i * 3 + 2] = (rings as u16 - 2) * segs + 2;
		}
	}

	unsafe { glutil::create_vertex_array_object(&verts, &inds, &attrib_offsets) }
}