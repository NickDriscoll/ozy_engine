use gl::types::*;
use std::collections::HashMap;
use std::mem;
use std::ptr;
use std::os::raw::c_void;
use crate::{glutil, io};
use glutil::ColorSpace;

const DEFAULT_TEX_PARAMS: [(GLenum, GLenum); 4] = [
	(gl::TEXTURE_WRAP_S, gl::REPEAT),
	(gl::TEXTURE_WRAP_T, gl::REPEAT),
	(gl::TEXTURE_MIN_FILTER, gl::LINEAR),
	(gl::TEXTURE_MAG_FILTER, gl::LINEAR)
];

pub const TEXTURE_MAP_COUNT: usize = 3;      //[albedo, normal, roughness]
const FLOATS_PER_TRANSFORM: usize = 16;

fn clip_from_screen(screen_size: glm::TVec2<u32>) -> glm::TMat4<f32> {
	glm::mat4(
		2.0 / screen_size.x as f32, 0.0, 0.0, -1.0,
		0.0, -(2.0 / screen_size.y as f32), 0.0, 1.0,
		0.0, 0.0, 1.0, 0.0,
		0.0, 0.0, 0.0, 1.0
	)
}

pub struct ScreenState {
    window_size: glm::TVec2<u32>,
    aspect_ratio: f32,
	default_framebuffer: Framebuffer,
	view_from_world: glm::TMat4<f32>,
    clipping_from_view: glm::TMat4<f32>,
    clipping_from_world: glm::TMat4<f32>,
    world_from_clipping: glm::TMat4<f32>,
    clipping_from_screen: glm::TMat4<f32>
}

impl ScreenState {
    pub fn new(window_size: glm::TVec2<u32>, view_from_world: glm::TMat4<f32>, clipping_from_view: glm::TMat4<f32>) -> Self {
        let aspect_ratio = window_size.x as f32 / window_size.y as f32;
        let clipping_from_world = clipping_from_view * view_from_world;
        let world_from_clipping = glm::affine_inverse(clipping_from_world);
        let clipping_from_screen = clip_from_screen(window_size);

        //Initialize default framebuffer
        let default_framebuffer = Framebuffer {
            name: 0,
            size: (window_size.x as GLsizei, window_size.y as GLsizei),
            clear_flags: gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT,
            cull_face: gl::BACK
        };

        ScreenState {
            window_size,
            aspect_ratio,
			default_framebuffer,
			view_from_world,
            clipping_from_view,
            clipping_from_world,
            world_from_clipping,
            clipping_from_screen
        }
	}
	
	pub fn update_view(&mut self, view_from_world: glm::TMat4<f32>) {
		let clipping_from_world = self.clipping_from_view * view_from_world;
        let world_from_clipping = glm::affine_inverse(clipping_from_world);
        let clipping_from_screen = clip_from_screen(self.window_size);
		
		self.view_from_world = view_from_world;
		self.clipping_from_world = clipping_from_world;
		self.world_from_clipping = world_from_clipping;
		self.clipping_from_screen = clipping_from_screen;
	}

	pub fn get_window_size(&self) -> glm::TVec2<u32> { self.window_size }
	pub fn get_aspect_ratio(&self) -> f32 { self.aspect_ratio }
	pub fn get_view_from_world(&self) -> &glm::TMat4<f32> { &self.view_from_world }
	pub fn get_clipping_from_view(&self) -> &glm::TMat4<f32> { &self.clipping_from_view }
	pub fn get_clipping_from_world(&self) -> &glm::TMat4<f32> { &self.clipping_from_world }
	pub fn get_world_from_clipping(&self) -> &glm::TMat4<f32> { &self.world_from_clipping }
	pub fn get_clipping_from_screen(&self) -> &glm::TMat4<f32> { &self.clipping_from_screen }
}

pub struct StaticGeometry {
    pub vao: GLuint,
    pub albedo: GLuint,
    pub normal: GLuint,
    pub model_matrix: glm::TMat4<f32>,
    pub index_count: GLsizei
}

//One contiguous piece of geometry
#[derive(Clone)]
pub struct SimpleMesh {
    pub vao: GLuint,
    pub index_count: GLint,
    pub origin: glm::TVec4<f32>,
    pub texture_maps: [GLuint; TEXTURE_MAP_COUNT]
}

impl SimpleMesh {
	pub fn new(vao: GLuint, index_count: GLint, material_name: &str, texture_keeper: &mut TextureKeeper, tex_params: &[(GLenum, GLenum)]) -> Self {
		let albedo = texture_keeper.fetch_texture(material_name, "albedo", tex_params, ColorSpace::Gamma);
        let normal = texture_keeper.fetch_texture(material_name, "normal", tex_params, ColorSpace::Linear);
        let roughness = texture_keeper.fetch_texture(material_name, "roughness", tex_params, ColorSpace::Linear);
		
		SimpleMesh {
			vao,
			index_count,
			origin: glm::vec4(0.0, 0.0, 0.0, 1.0),
			texture_maps: [albedo, normal, roughness]
		}
	}

	pub unsafe fn draw(&self) {
        gl::BindVertexArray(self.vao);
		gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_SHORT, ptr::null());
	}

    pub fn from_ozy(path: &str, texture_keeper: &mut TextureKeeper, tex_params: &[(GLenum, GLenum)]) -> Self {
        match io::OzyMesh::load(path) {
            Some(meshdata) => unsafe {
                let vao = glutil::create_vertex_array_object(&meshdata.vertex_array.vertices, &meshdata.vertex_array.indices, &meshdata.vertex_array.attribute_offsets);
                let count = meshdata.geo_boundaries[1] as GLint;
                let origin = meshdata.origins[0];
                let albedo = texture_keeper.fetch_texture(&meshdata.texture_names[0], "albedo", tex_params, ColorSpace::Gamma);
                let normal = texture_keeper.fetch_texture(&meshdata.texture_names[0], "normal", tex_params, ColorSpace::Linear);
                let roughness = texture_keeper.fetch_texture(&meshdata.texture_names[0], "roughness", tex_params, ColorSpace::Linear);
    
                SimpleMesh {
                    vao,
                    origin,
                    texture_maps: [albedo, normal, roughness],
                    index_count: count as GLint
                }
            }
            None => {
                panic!("Unable to load OzyMesh: {}", path);
            }
        }
    }
}

#[derive(Clone)]
pub struct InstancedMesh {
    vao: GLuint,
    transform_buffer: GLuint,
    index_count: GLint,
    active_instances: usize,
	max_instances: usize,
	texture_maps: [GLuint; TEXTURE_MAP_COUNT]
}

impl InstancedMesh {
    pub unsafe fn new(vao: GLuint, index_count: GLint, max_instances: usize, instanced_attribute: GLuint, maps: [GLuint; TEXTURE_MAP_COUNT]) -> Self {
		//Create GPU buffer for instanced matrices
		let transform_buffer = create_transform_buffer(vao, max_instances, instanced_attribute);        
        
        InstancedMesh {
            vao,
            max_instances,
            index_count,
            active_instances: 0,
			transform_buffer,
			texture_maps: maps
        }
	}

	pub unsafe fn from_simplemesh(s_mesh: &SimpleMesh, max_instances: usize, instanced_attribute: GLuint) -> Self {
		Self::new(s_mesh.vao, s_mesh.index_count, max_instances, instanced_attribute, s_mesh.texture_maps)
	}

    pub unsafe fn draw(&self) {
        gl::BindVertexArray(self.vao);
		gl::DrawElementsInstanced(gl::TRIANGLES, self.index_count, gl::UNSIGNED_SHORT, ptr::null(), self.active_instances as GLint);
    }

	pub fn max_instances(&self) -> usize { self.max_instances }
	pub fn texture_maps(&self) -> &[GLuint; TEXTURE_MAP_COUNT] { &self.texture_maps }

    pub fn update_buffer(&mut self, transforms: &[f32]) {
        //Record the current active instance count
        self.active_instances = transforms.len() / FLOATS_PER_TRANSFORM;

        //Update GPU buffer storing hit volume transforms
		if transforms.len() > 0 {
			unsafe {
				gl::BindBuffer(gl::ARRAY_BUFFER, self.transform_buffer);
				gl::BufferSubData(gl::ARRAY_BUFFER,
								0 as GLsizeiptr,
								(transforms.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
								&transforms[0] as *const GLfloat as *const c_void
								);
			}
		}
    }
}

unsafe fn create_transform_buffer(vao: GLuint, max_instances: usize, instanced_attribute: GLuint) -> GLuint {
	gl::BindVertexArray(vao);

	let mut b = 0;
	gl::GenBuffers(1, &mut b);
	gl::BindBuffer(gl::ARRAY_BUFFER, b);
	gl::BufferData(gl::ARRAY_BUFFER, (max_instances * FLOATS_PER_TRANSFORM * mem::size_of::<GLfloat>()) as GLsizeiptr, ptr::null(), gl::DYNAMIC_DRAW);

	//Attach this buffer to the shell_mesh vao
	//We have to individually bind each column of the matrix as a different vec4 vertex attribute
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

	b
}

pub struct TextureKeeper {
    map: HashMap<String, u32>
}

impl TextureKeeper {
    pub fn new() -> Self {
        TextureKeeper {
            map: HashMap::new()
        }
    }

    pub fn fetch_texture(&mut self, name: &str, map_type: &str, tex_params: &[(GLenum, GLenum)], color_space: glutil::ColorSpace) -> GLuint {
		let texture_path = format!("materials/{}/{}.png", name, map_type);
		match self.map.get(&texture_path) {
			Some(t) => { *t }
			None => {
				let name = unsafe { glutil::load_texture(&texture_path, tex_params, color_space) };
				self.map.insert(texture_path, name);
				name
			}
        }
    }
}

pub struct Framebuffer {
    pub name: GLuint,
    pub size: (GLsizei, GLsizei),
    pub clear_flags: GLenum,
    pub cull_face: GLenum
}

impl Framebuffer {
    pub unsafe fn bind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.name);
        gl::Viewport(0, 0, self.size.0, self.size.1);
        gl::Clear(self.clear_flags);
        gl::CullFace(self.cull_face);
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.name);
        }
    }
}

//A framebuffer object with color and depth attachments
pub struct RenderTarget {
    pub framebuffer: Framebuffer,
    pub texture: GLuint
}

impl RenderTarget {
    pub unsafe fn new(size: (GLint, GLint)) -> Self {
        let mut fbo = 0;
		let mut texs = [0; 2];
		gl::GenFramebuffers(1, &mut fbo);
		gl::GenTextures(2, &mut texs[0]);
		let (color_tex, depth_tex) = (texs[0], texs[1]);

		//Initialize the color buffer
		gl::BindTexture(gl::TEXTURE_2D, color_tex);
		gl::TexImage2D(
			gl::TEXTURE_2D,
			0,
			gl::SRGB8_ALPHA8 as GLint,
			size.0,
			size.1,
			0,
			gl::RGBA,
			gl::FLOAT,
			ptr::null()
		);
		let params = [
			(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE),
			(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE),
			(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR),
			(gl::TEXTURE_MAG_FILTER, gl::NEAREST)
		];
        glutil::apply_texture_parameters(&params);
	    gl::GenerateMipmap(gl::TEXTURE_2D);

		gl::BindTexture(gl::TEXTURE_2D, depth_tex);
		gl::TexImage2D(
			gl::TEXTURE_2D,
			0,
			gl::DEPTH_COMPONENT as GLint,
			size.0,
			size.1,
			0,
			gl::DEPTH_COMPONENT,
			gl::FLOAT,
			ptr::null()
		);
		let params = [
			(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE),
			(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE),
			(gl::TEXTURE_MIN_FILTER, gl::NEAREST),
			(gl::TEXTURE_MAG_FILTER, gl::NEAREST)
		];
		glutil::apply_texture_parameters(&params);

		gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
		gl::FramebufferTexture2D(
			gl::FRAMEBUFFER,
			gl::COLOR_ATTACHMENT0,
			gl::TEXTURE_2D,
			color_tex,
			0
		);
		gl::FramebufferTexture2D(
			gl::FRAMEBUFFER,
			gl::DEPTH_ATTACHMENT,
			gl::TEXTURE_2D,
			depth_tex,
			0
		);
		gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

		let f_buffer = Framebuffer {
			name: fbo,
			size: (size.0 as GLsizei, size.1 as GLsizei),
			clear_flags: gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT,
			cull_face: gl::BACK
		};

		RenderTarget {
			framebuffer: f_buffer,
			texture: color_tex
		}
    }

    pub unsafe fn new_shadow(size: (GLint, GLint)) -> Self {
        let mut shadow_framebuffer = 0;
		let mut shadow_texture = 0;

		gl::GenFramebuffers(1, &mut shadow_framebuffer);
		gl::GenTextures(1, &mut shadow_texture);

		//Initialize the texture
		gl::BindTexture(gl::TEXTURE_2D, shadow_texture);
		gl::TexImage2D(
			gl::TEXTURE_2D,
			0,
			gl::DEPTH_COMPONENT as GLint,
			size.0,
			size.1,
			0,
			gl::DEPTH_COMPONENT,
			gl::FLOAT,
			ptr::null()
		);
		glutil::apply_texture_parameters(&DEFAULT_TEX_PARAMS);

		gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_framebuffer);
		gl::FramebufferTexture2D(
			gl::FRAMEBUFFER,
			gl::DEPTH_ATTACHMENT,
			gl::TEXTURE_2D,
			shadow_texture,
			0
		);
		gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

		let framebuffer = Framebuffer {
			name: shadow_framebuffer,
			size: (size.0, size.1),
			clear_flags: gl::DEPTH_BUFFER_BIT,
			cull_face: gl::BACK
		};

		RenderTarget {
			framebuffer,
			texture: shadow_texture
		}
	}

    pub unsafe fn bind(&self) { self.framebuffer.bind(); }

    pub unsafe fn resize(&mut self, size: (u32, u32)) {
        gl::DeleteFramebuffers(1, &self.framebuffer.name);
        let n = Self::new((size.0 as GLint, size.1 as GLint));
        self.framebuffer = n.framebuffer;
        self.texture = n.texture;
    }
}