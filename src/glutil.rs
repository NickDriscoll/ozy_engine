use gl::types::*;
use std::ffi::CString;
use std::str;
use std::io;
use std::io::Read;
use std::fs::File;
use std::{mem, process, ptr};
use std::os::raw::c_void;
use image::DynamicImage;
use crate::structs::*;

const FLOATS_PER_TRANSFORM: usize = 16;

pub extern "system" fn gl_debug_callback(source: GLenum, gltype: GLenum, id: GLuint, severity: GLenum, length: GLsizei, message: *const GLchar, _: *mut c_void) {
	println!("--------------------OpenGL debug message--------------------");
	println!("ID: {}", id);
	
	match source {
		gl::DEBUG_SOURCE_API => 				{ println!("Source: API"); }
		gl::DEBUG_SOURCE_WINDOW_SYSTEM => 		{ println!("Source: Window System"); }
		gl::DEBUG_SOURCE_SHADER_COMPILER => 	{ println!("Source: Shader Compiler"); }
		gl::DEBUG_SOURCE_THIRD_PARTY => 		{ println!("Source: Third Party"); }
		gl::DEBUG_SOURCE_APPLICATION => 		{ println!("Source: Application"); }
		gl::DEBUG_SOURCE_OTHER => 				{ println!("Source: Other"); }
		_ => {}
	}

	match gltype {
		gl::DEBUG_TYPE_ERROR => 					{ println!("Type: Error") }
		gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => 		{ println!("Type: Deprecated Behaviour") }
		gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => 		{ println!("Type: Undefined Behaviour") }
		gl::DEBUG_TYPE_PORTABILITY => 				{ println!("Type: Portability") }
		gl::DEBUG_TYPE_PERFORMANCE => 				{ println!("Type: Performance") }
		gl::DEBUG_TYPE_MARKER => 					{ println!("Type: Marker") }
		gl::DEBUG_TYPE_PUSH_GROUP => 				{ println!("Type: Push Group") }
		gl::DEBUG_TYPE_POP_GROUP => 				{ println!("Type: Pop Group") }
		gl::DEBUG_TYPE_OTHER => 					{ println!("Type: Other") }
		_ => {}
	}

	match severity {
		gl::DEBUG_SEVERITY_HIGH => { 
			println!("Severity: High"); 
		}
		gl::DEBUG_SEVERITY_MEDIUM => { 
			println!("Severity: Medium"); 
		}
		gl::DEBUG_SEVERITY_LOW => { 
			println!("Severity: Low"); 
		}
		gl::DEBUG_SEVERITY_NOTIFICATION => { 
			println!("Severity: Notification"); 
		}
		_ => {}
	}

	let m = unsafe {
		let mut buffer = vec![0; length as usize];
		for i in 0..length as isize {
			buffer[i as usize] = *message.offset(i) as u8;
		}
		String::from_utf8(buffer).unwrap()
	};

	println!("Message: {}", m);
	println!("------------------------------------------------------------");
}

pub unsafe fn compile_shader(shadertype: GLenum, source: &str) -> GLuint {
	let shader = gl::CreateShader(shadertype);
	let cstr_vert = CString::new(source.as_bytes()).unwrap();
	gl::ShaderSource(shader, 1, &cstr_vert.as_ptr(), ptr::null());
	gl::CompileShader(shader);

	//Check for errors
	let mut success = gl::FALSE as GLint;
	let mut log_size = 0;
	gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
	gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_size);
	let mut infolog = Vec::with_capacity(log_size as usize);
	if success != gl::TRUE as GLint {
		gl::GetShaderInfoLog(shader, log_size, ptr::null_mut(), infolog.as_mut_ptr() as *mut GLchar);
		shader_compilation_error(&infolog);
	}
	shader
}

pub unsafe fn compile_shader_from_file(shadertype: GLenum, path: &str) -> Result<GLuint, io::Error> {
	let mut source = String::new();

	match File::open(path) {
		Ok(mut file) => {
			file.read_to_string(&mut source).unwrap();
		}
		Err(e) => { return Err(e); }
	}
	Ok(compile_shader(shadertype, &source))
}

pub fn compile_program_from_files(vertex_name: &str, fragment_name: &str) -> Result<GLuint, io::Error> {
	unsafe {
		let vertexshader = match compile_shader_from_file(gl::VERTEX_SHADER, vertex_name) {
			Ok(shader) => { shader }
			Err(e) => { return Err(e); }
		};

		let fragmentshader = match compile_shader_from_file(gl::FRAGMENT_SHADER, fragment_name) {
			Ok(shader) => { shader }
			Err(e) => { return Err(e); }
		};

		//Link shaders
		let shader_progam = gl::CreateProgram();
		gl::AttachShader(shader_progam, vertexshader);
		gl::AttachShader(shader_progam, fragmentshader);
		gl::LinkProgram(shader_progam);

		//Check for errors
		let mut success = gl::FALSE as GLint;
		let mut log_size = 0;
		gl::GetProgramiv(shader_progam, gl::LINK_STATUS, &mut success);
		gl::GetProgramiv(shader_progam, gl::INFO_LOG_LENGTH, &mut log_size);
		let mut infolog = vec![0; log_size as usize];
		if success != gl::TRUE as GLint {
			gl::GetProgramInfoLog(shader_progam, log_size, ptr::null_mut(), infolog.as_mut_ptr() as *mut GLchar);
			shader_compilation_error(&infolog);
		}

		gl::DeleteShader(vertexshader);
		gl::DeleteShader(fragmentshader);
		Ok(shader_progam)
	}
}

fn shader_compilation_error(infolog: &[u8]) {
	let error_message = match str::from_utf8(infolog) {
		Ok(message) => { message }
		Err(_) => { panic!("Error getting the shader compilation error. This statement should be unreachable."); }
	};
	panic!("\n--------SHADER COMPILATION ERROR--------\n{}", error_message);
}

pub unsafe fn uniform_location(program: GLuint, name: &str) -> GLint {
	let cstring = CString::new(name.as_bytes()).unwrap();
	gl::GetUniformLocation(program, cstring.as_ptr())
}

pub unsafe fn gl_gen_buffer() -> GLuint {
	let mut buffer = 0;
	gl::GenBuffers(1, &mut buffer);
	buffer
}

//Input: array of vertex data, an array of indices, and an array representing the number of elements per vertex attribute
//Output: A vertex array object with the vertex data bound as a GL_ARRAY_BUFFER, and the index data bound as a GL_ELEMENT_ARRAY_BUFFER
pub unsafe fn create_vertex_array_object(vertices: &[f32], indices: &[u16], attribute_strides: &[i32]) -> GLuint {
	let mut vao = 0;
	gl::GenVertexArrays(1, &mut vao);
	let (vbo, ebo) = {
		let mut s = [0; 2];
		gl::GenBuffers(2, &mut s[0]);
		(s[0], s[1])
	};

	gl::BindVertexArray(vao);

	gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
	gl::BufferData(gl::ARRAY_BUFFER,
				   (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
				   &vertices[0] as *const f32 as *const c_void,
				   gl::STATIC_DRAW);

	gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
	gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
				   (indices.len() * mem::size_of::<GLushort>()) as GLsizeiptr,
				   &indices[0] as *const u16 as *const c_void,
				   gl::STATIC_DRAW);

	//Calculate the stride in bytes between individual vertices
	let byte_stride = {
		let mut sum = 0;
		for stride in attribute_strides {
			sum += stride;
		}
		sum * mem::size_of::<GLfloat>() as i32
	};

	//Configure and enable the vertex attributes
	let mut cumulative_size = 0;
	for i in 0..attribute_strides.len() {
		gl::VertexAttribPointer(i as GLuint,
								attribute_strides[i],
								gl::FLOAT,
								gl::FALSE,
								byte_stride,
								(cumulative_size * mem::size_of::<GLfloat>() as GLuint) as *const c_void);
		
		gl::EnableVertexAttribArray(i as GLuint);
		cumulative_size += attribute_strides[i] as u32;
	}

	vao
}

pub fn load_texture(path: &str, parameters: &[(GLenum, GLenum)], color_space: ColorSpace) -> GLuint {
	unsafe {
		load_texture_from_data(image_data_from_path(path, color_space), parameters)
	}
}

pub enum ColorSpace {
	Linear,
	Gamma
}

pub fn image_data_from_path(path: &str, space: ColorSpace) -> ImageData {
	match image::open(path) {
		Ok(DynamicImage::ImageRgb8(im)) => {
			let internal_format = match space {
				ColorSpace::Linear => { gl::RGB8 }
				ColorSpace::Gamma => { gl::SRGB8 }
			};

			let width = im.width();
			let height = im.height();
			let raw = im.into_raw();
			ImageData {
				data: raw,
				width: width as GLint,
				height: height as GLint,
				format: gl::RGB,
				internal_format
			}
		}
		Ok(DynamicImage::ImageRgba8(im)) => {
			let internal_format = match space {
				ColorSpace::Linear => { gl::RGBA8 }
				ColorSpace::Gamma => { gl::SRGB8_ALPHA8 }
			};

			let width = im.width();
			let height = im.height();
			let raw = im.into_raw();
			ImageData {
				data: raw,
				width: width as GLint,
				height: height as GLint,
				format: gl::RGBA,
				internal_format
			}
		}
		Ok(DynamicImage::ImageRgba16(im)) => {
			let internal_format = match space {
				ColorSpace::Linear => { gl::RGBA8 }
				ColorSpace::Gamma => { gl::SRGB8_ALPHA8 }
			};

			let width = im.width();
			let height = im.height();
			let raw = im.into_raw();
			let mut rawu8 = vec![0u8; raw.len()];
			for i in 0..raw.len() {
				rawu8[i] = raw[i] as u8;
			}

			ImageData {
				data: rawu8,
				width: width as GLint,
				height: height as GLint,
				format: gl::RGBA,
				internal_format
			}
		}
		Ok(DynamicImage::ImageLuma8(im)) => {
			let width = im.width();
			let height = im.height();
			let raw = im.into_raw();

			ImageData {
				data: raw,
				width: width as GLint,
				height: height as GLint,
				format: gl::RED,
				internal_format: gl::R8
			}
		}
		Ok(_) => {
            println!("{} is of an unsupported image type", path);
            ImageData {
                data: vec![0u8],
                width: 1,
                height: 1,
                format: gl::RGBA,
                internal_format: gl::SRGB8_ALPHA8
            }
		}
		Err(e) => {
			println!("Unable to open {}: {}", path, e);
            ImageData {
                data: vec![0u8],
                width: 1,
                height: 1,
                format: gl::RGBA,
                internal_format: gl::SRGB8_ALPHA8
            }
		}
	}
}

//Create and attaches an instanced array buffer of 4x4 homogenous matrices of size max_instances to vao at instanced_attribute
//Returns the name of the new buffer
pub unsafe fn create_instanced_transform_buffer(vao: GLuint, max_instances: usize, instanced_attribute: GLuint) -> GLuint {
	gl::BindVertexArray(vao);

	let data = vec![0.0f32; max_instances * FLOATS_PER_TRANSFORM];
	let mut b = 0;
	gl::GenBuffers(1, &mut b);
	gl::BindBuffer(gl::ARRAY_BUFFER, b);
	gl::BufferData(gl::ARRAY_BUFFER, (max_instances * FLOATS_PER_TRANSFORM * mem::size_of::<GLfloat>()) as GLsizeiptr, &data[0] as *const f32 as *const c_void, gl::DYNAMIC_DRAW);

	//Attach this buffer to the shell_mesh vao
	//We have to individually bind each column of the matrix as a different vec4 vertex attribute
	bind_new_transform_buffer(instanced_attribute);

	b
}

pub unsafe fn bind_new_transform_buffer(instanced_attribute: GLuint) {	
	for i in 0..4 {
		let attribute_index = instanced_attribute + i;
		gl::VertexAttribPointer(attribute_index,
								4,
								gl::FLOAT,
								gl::FALSE,
								(FLOATS_PER_TRANSFORM * mem::size_of::<GLfloat>()) as GLsizei,
								(i * 4 * mem::size_of::<GLfloat>() as GLuint) as *const c_void);
		gl::EnableVertexAttribArray(attribute_index);
		gl::VertexAttribDivisor(attribute_index, 1);
	}
}

//Apllies the list of parameters to the current bound 2D texture
pub unsafe fn apply_texture_parameters(parameters: &[(GLenum, GLenum)]) {
	for param in parameters {
		gl::TexParameteri(gl::TEXTURE_2D, param.0, param.1 as GLint);
	}
}

pub unsafe fn load_texture_from_data(image_data: ImageData, parameters: &[(GLenum, GLenum)]) -> GLuint {
	//Create texture
	let mut tex = 0;
	gl::GenTextures(1, &mut tex);
	gl::BindTexture(gl::TEXTURE_2D, tex);

	//Apply texture parameters
	apply_texture_parameters(parameters);

	//Upload texture data
	gl::TexImage2D(gl::TEXTURE_2D,
				   0,
				   image_data.internal_format as i32,
				   image_data.width,
				   image_data.height,
				   0,
				   image_data.format,
				   gl::UNSIGNED_BYTE,
				   &image_data.data[0] as *const u8 as *const c_void);
	gl::GenerateMipmap(gl::TEXTURE_2D);	//Generate mipmaps
	tex
}

pub unsafe fn bind_matrix4(program: GLuint, name: &str, matrix: &glm::TMat4<f32>) {
	gl::UseProgram(program);
	gl::UniformMatrix4fv(uniform_location(program, name), 1, gl::FALSE, &glm::value_ptr(matrix)[0]);
}

pub unsafe fn bind_matrix4_array(program: GLuint, name: &str, matrices: &[glm::TMat4<f32>]) {
	gl::UseProgram(program);
	gl::UniformMatrix4fv(uniform_location(program, name), matrices.len() as GLint, gl::FALSE, &glm::value_ptr(&matrices[0])[0]);
}

pub unsafe fn bind_vector4(program: GLuint, name: &str, vector: &glm::TVec4<f32>) {
	gl::UseProgram(program);
	gl::Uniform4fv(uniform_location(program, name), 1, &[vector.x, vector.y, vector.z, vector.w] as *const GLfloat);
}

pub unsafe fn bind_vector3(program: GLuint, name: &str, vector: &glm::TVec3<f32>) {
	gl::UseProgram(program);
	gl::Uniform3fv(uniform_location(program, name), 1, &[vector.x, vector.y, vector.z] as *const GLfloat);
}

pub unsafe fn bind_vector2(program: GLuint, name: &str, vector: &glm::TVec2<f32>) {
	gl::UseProgram(program);
	gl::Uniform2fv(uniform_location(program, name), 1, &[vector.x, vector.y] as *const GLfloat);
}

pub unsafe fn bind_int(program: GLuint, name: &str, number: GLint) {
	gl::UseProgram(program);
	gl::Uniform1i(uniform_location(program, name), number);
}

pub unsafe fn bind_float(program: GLuint, name: &str, number: GLfloat) {
	gl::UseProgram(program);
	gl::Uniform1f(uniform_location(program, name), number);
}

pub unsafe fn bind_float_array(program: GLuint, name: &str, array: &[f32]) {
	gl::UseProgram(program);
	gl::Uniform1fv(uniform_location(program, name), array.len() as GLint, &array[0] as *const GLfloat);
}