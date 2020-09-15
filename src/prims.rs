fn sphere_index_count(segments: usize, rings: usize) -> usize {	
	//segments * (rings - 2) * 6 + 6 * segments
	segments * (rings - 2) * 6
}

unsafe fn sphere_vao(radius: f32, segments: usize, rings: usize) -> GLuint {
	let mut verts = vec![0.0; segments * (rings - 1) + 2];
	let mut inds = vec![0u16; sphere_index_count(segments, rings)];

	//Compute the vertices of the sphere
	for i in 0..(rings - 1) {
		let r = i as f32 + 1.0;
		for j in 0..segments {
			let s = j as f32;
			let theta = glm::two_pi::<f32>() * s / segments as f32;
			let phi = glm::pi::<f32>() * r / rings as f32;

			let vert = 3 * (i * segments + j);
			verts[vert] = radius * f32::cos(theta) * f32::sin(phi);
			verts[vert + 1] = radius * f32::sin(theta) * f32::sin(phi);
			verts[vert + 2] = radius * f32::cos(phi);
		}
	}

	//Compute sphere index data
	let segs = segments as u16;
	for i in 0..(rings - 2) {
		let offset = i * segments;
		for j in 0..segs {
			let ind = 6 * (offset + j as usize);
			inds[ind] = offset as u16 + j;
			inds[ind + 2] = offset as u16 + j + segs;
			inds[ind + 3] = inds[ind + 2] + 1;
			inds[ind + 4] = inds[ind + 2];

			if j == segs - 1 {
				inds[ind + 1] = offset as u16;
			} else {
				inds[ind + 1] = j + 1;
			}
			inds[ind + 5] = inds[ind + 1];
		}
	}

	glutil::create_vertex_array_object(&verts, &inds, &[3])
}