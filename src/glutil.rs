use gl::types::*;
use std::ffi::CString;
use std::str;
use std::io::Read;
use std::fs::File;
use std::{mem, process, ptr};
use std::os::raw::c_void;
use image::DynamicImage;
use crate::structs::*;
use crate::routines::*;

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

pub unsafe fn compile_shader_from_file(shadertype: GLenum, path: &str) -> GLuint {
	let mut source = String::new();

	match File::open(path) {
		Ok(mut file) => {
			file.read_to_string(&mut source).unwrap();
		}
		Err(e) => {
			println!("{}\npath: \"{}\"", e, path);
			process::exit(-1);
		}
	}
	compile_shader(shadertype, &source)
}

pub unsafe fn compile_program_from_files(vertex_name: &str, fragment_name: &str) -> GLuint {
	let vertexshader = compile_shader_from_file(gl::VERTEX_SHADER, vertex_name);
	let fragmentshader = compile_shader_from_file(gl::FRAGMENT_SHADER, fragment_name);

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
	shader_progam
}

pub fn shader_compilation_error(infolog: &[u8]) {
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
	let vbo = gl_gen_buffer();
	let ebo = gl_gen_buffer();

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

pub unsafe fn load_texture(path: &str, parameters: &[(GLenum, GLenum)]) -> GLuint {
	load_texture_from_data(image_data_from_path(path), parameters)
}

pub fn image_data_from_path(path: &str) -> ImageData {
	match image::open(path) {
		Ok(DynamicImage::ImageRgb8(im)) => {
			let width = im.width();
			let height = im.height();
			let raw = im.into_raw();
			ImageData {
				data: raw,
				width: width as GLint,
				height: height as GLint,
				format: gl::RGB,
				internal_format: gl::SRGB
			}
		}
		Ok(DynamicImage::ImageRgba8(im)) => {
			let width = im.width();
			let height = im.height();
			let raw = im.into_raw();
			ImageData {
				data: raw,
				width: width as GLint,
				height: height as GLint,
				format: gl::RGBA,
				internal_format: gl::SRGB8_ALPHA8
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

pub unsafe fn load_texture_from_data(image_data: ImageData, parameters: &[(GLenum, GLenum)]) -> GLuint {
	//Create texture
	let mut tex = 0;
	gl::GenTextures(1, &mut tex);
	gl::BindTexture(gl::TEXTURE_2D, tex);

	//Apply texture parameters
	for param in parameters {
		gl::TexParameteri(gl::TEXTURE_2D, param.0, param.1 as GLint);
	}

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
	gl::UniformMatrix4fv(uniform_location(program, name), 1, gl::FALSE, &flatten_glm(matrix) as *const GLfloat);
}

pub unsafe fn bind_vector4(program: GLuint, name: &str, vector: &glm::TVec4<f32>) {
	gl::Uniform4fv(uniform_location(program, name), 1, &[vector.x, vector.y, vector.z, vector.w] as *const GLfloat);
}

pub unsafe fn bind_byte(program: GLuint, name: &str, byte: GLint) {
	gl::Uniform1i(uniform_location(program, name), byte);
}