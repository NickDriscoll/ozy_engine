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
	fov_radians: f32,
	default_framebuffer: Framebuffer,
	view_from_world: glm::TMat4<f32>,
    clipping_from_view: glm::TMat4<f32>,
    clipping_from_world: glm::TMat4<f32>,
    world_from_clipping: glm::TMat4<f32>,
	world_from_view: glm::TMat4<f32>,
    clipping_from_screen: glm::TMat4<f32>
}

impl ScreenState {
    pub fn new(window_size: glm::TVec2<u32>, view_from_world: glm::TMat4<f32>, fov_radians: f32, near: f32, far: f32) -> Self {
		let clipping_from_view = glm::perspective_zo(window_size.x as f32 / window_size.y as f32, fov_radians, near, far);
        let aspect_ratio = window_size.x as f32 / window_size.y as f32;
        let clipping_from_world = clipping_from_view * view_from_world;
        let world_from_clipping = glm::affine_inverse(clipping_from_world);
		let world_from_view = glm::affine_inverse(view_from_world);
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
			fov_radians,
			default_framebuffer,
			view_from_world,
            clipping_from_view,
            clipping_from_world,
            world_from_clipping,
			world_from_view,
            clipping_from_screen
        }
	}
	
	pub fn update_view(&mut self, view_from_world: glm::TMat4<f32>) {
		let clipping_from_world = self.clipping_from_view * view_from_world;
        let world_from_clipping = glm::affine_inverse(clipping_from_world);
		let world_from_view = glm::affine_inverse(view_from_world);
        let clipping_from_screen = clip_from_screen(self.window_size);
		
		self.view_from_world = view_from_world;
		self.clipping_from_world = clipping_from_world;
		self.world_from_clipping = world_from_clipping;
		self.world_from_view = world_from_view;
		self.clipping_from_screen = clipping_from_screen;
	}

	pub fn get_fov_radians(&self) -> f32 { self.fov_radians }
	pub fn get_window_size(&self) -> glm::TVec2<u32> { self.window_size }
	pub fn get_aspect_ratio(&self) -> f32 { self.aspect_ratio }
	pub fn get_view_from_world(&self) -> &glm::TMat4<f32> { &self.view_from_world }
	pub fn get_clipping_from_view(&self) -> &glm::TMat4<f32> { &self.clipping_from_view }
	pub fn get_clipping_from_world(&self) -> &glm::TMat4<f32> { &self.clipping_from_world }
	pub fn get_world_from_clipping(&self) -> &glm::TMat4<f32> { &self.world_from_clipping }
	pub fn get_world_from_view(&self) -> &glm::TMat4<f32> { &self.world_from_view }
	pub fn get_clipping_from_screen(&self) -> &glm::TMat4<f32> { &self.clipping_from_screen }
}

pub struct StaticGeometry {
    pub vao: GLuint,
    pub albedo: GLuint,
    pub normal: GLuint,
    pub model_matrix: glm::TMat4<f32>,
    pub index_count: GLsizei
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
	
	pub unsafe fn new_multisampled(size: (GLint, GLint), samples: GLint) -> Self {
        let mut fbo = 0;
		let mut texs = [0; 2];
		gl::GenFramebuffers(1, &mut fbo);
		gl::GenTextures(2, &mut texs[0]);
		let (color_tex, depth_tex) = (texs[0], texs[1]);

		//Initialize the color buffer
		gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, color_tex);
		gl::TexImage2DMultisample(
			gl::TEXTURE_2D_MULTISAMPLE,
			samples,
			gl::SRGB8_ALPHA8,
			size.0,
			size.1,
			gl::TRUE
		);
		let params = [
			(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE),
			(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE),
			(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR),
			(gl::TEXTURE_MAG_FILTER, gl::NEAREST)
		];
        glutil::apply_texture_parameters(&params);
	    //gl::GenerateMipmap(gl::TEXTURE_2D);

		gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, depth_tex);
		gl::TexImage2DMultisample(
			gl::TEXTURE_2D_MULTISAMPLE,
			samples,
			gl::DEPTH_COMPONENT,
			size.0,
			size.1,
			gl::TRUE
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
			gl::TEXTURE_2D_MULTISAMPLE,
			color_tex,
			0
		);
		gl::FramebufferTexture2D(
			gl::FRAMEBUFFER,
			gl::DEPTH_ATTACHMENT,
			gl::TEXTURE_2D_MULTISAMPLE,
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