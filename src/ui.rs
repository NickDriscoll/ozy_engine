use crate::structs::{OptionVec};
use crate::glutil;
use crate::render;
use glyph_brush::{BrushAction, BrushError, ab_glyph::FontArc, GlyphBrush, GlyphBrushBuilder, GlyphCruncher, GlyphVertex, ab_glyph::PxScale, Section, Rectangle, Text};
use gl::types::*;
use std::os::raw::c_void;
use std::{mem, ptr};

const FLOATS_PER_GLYPH: usize = 32;
type GlyphBrushVertexType = [f32; FLOATS_PER_GLYPH];

fn insert_index_buffer_quad(index_buffer: &mut [u16], i: usize) {
	index_buffer[i * 6] = 4 * i as u16;
	index_buffer[i * 6 + 1] = index_buffer[i * 6] + 1;
	index_buffer[i * 6 + 2] = index_buffer[i * 6] + 2;
	index_buffer[i * 6 + 3] = index_buffer[i * 6] + 3;
	index_buffer[i * 6 + 4] = index_buffer[i * 6] + 2;
	index_buffer[i * 6 + 5] = index_buffer[i * 6] + 1;
}

//First argument to glyph_brush.process_queued()
unsafe fn upload_glyph_texture(glyph_texture: GLuint, rect: Rectangle<u32>, data: &[u8]) {
	gl::TextureSubImage2D(
		glyph_texture,
		0,
		rect.min[0] as _,
		rect.min[1] as _,
		rect.width() as _,
		rect.height() as _,
		gl::RED,
		gl::UNSIGNED_BYTE,
		data.as_ptr() as _
	);
}

//Second argument to glyph_brush.process_queued()
fn glyph_vertex_transform(vertex: GlyphVertex) -> GlyphBrushVertexType {
	let left = vertex.pixel_coords.min.x as f32;
	let right = vertex.pixel_coords.max.x as f32;
	let top = vertex.pixel_coords.min.y as f32;
	let bottom = vertex.pixel_coords.max.y as f32;
	let texleft = vertex.tex_coords.min.x;
	let texright = vertex.tex_coords.max.x;
	let textop = vertex.tex_coords.min.y;
	let texbottom = vertex.tex_coords.max.y;

	//We need to return four vertices in screen space
	[
		left, bottom, texleft, texbottom, vertex.extra.color[0], vertex.extra.color[1], vertex.extra.color[2], vertex.extra.color[3],
		right, bottom, texright, texbottom, vertex.extra.color[0], vertex.extra.color[1], vertex.extra.color[2], vertex.extra.color[3],
		left, top, texleft, textop, vertex.extra.color[0], vertex.extra.color[1], vertex.extra.color[2], vertex.extra.color[3],
		right, top, texright, textop, vertex.extra.color[0], vertex.extra.color[1], vertex.extra.color[2], vertex.extra.color[3]
	]	
}

unsafe fn draw_ui_elements(vao: GLuint, shader: GLuint, count: usize, clipping_from_screen: &glm::TMat4<f32>) {
    gl::UseProgram(shader);
	glutil::bind_matrix4(shader, "clipping_from_screen", &clipping_from_screen);
	gl::BindVertexArray(vao);
	gl::DrawElements(gl::TRIANGLES, 6 * count as GLint, gl::UNSIGNED_SHORT, ptr::null());
}

//Subset of UIState created to fix some borrowing issues
pub struct UIInternals<'a, T> {
    vao_flag: bool,
	pub glyph_brush: GlyphBrush<GlyphBrushVertexType>,
	window_size: (u32, u32),
    buttons: OptionVec<UIButton<T>>,
    sections: OptionVec<Section<'a>>
}

impl<'a, T> UIInternals<'a, T> {
    pub fn new(glyph_brush: GlyphBrush<GlyphBrushVertexType>, window_size: (u32, u32)) -> Self {
        UIInternals {
            vao_flag: false,
			glyph_brush,
			window_size,
            buttons: OptionVec::new(),
            sections: OptionVec::new()
        }
    }

    pub fn add_button(&mut self, button: UIButton<T>) -> usize {
        self.vao_flag = true;
        self.buttons.insert(button)
    }

    pub fn add_section(&mut self, section: Section<'a>) -> usize {
        self.vao_flag = true;
        self.sections.insert(section)
    }

    pub fn delete_button(&mut self, index: usize) {
        self.vao_flag = true;
        if let Some(button) = &self.buttons[index] {
            self.sections.delete(button.section_id());
            self.buttons.delete(index);
        }
    }

    pub fn delete_section(&mut self, index: usize) {
        self.vao_flag = true;
        self.sections.delete(index);
    }
}

pub struct UIState<'a, T> {
    pub button_color_buffer: GLuint,
    pub buttons_vao: Option<GLuint>,
    pub internals: UIInternals<'a, T>,
    pub glyph_texture: GLuint,
    pub glyph_vao: Option<GLuint>,
	pub glyph_count: usize,
	menu_chains: Vec<Vec<usize>>, //Array of array of menu ids used for nested menu traversal
	menus: Vec<Menu<'a, T>>,
	text_elements: Vec<UIText<'a>>,
	programs: [GLuint; 2],
	hidden_menus: Option<Vec<usize>>
}

impl<'a, T: Copy> UIState<'a, T> {
	const BUTTON_SHADER: usize = 0;
	const GLYPH_SHADER: usize = 1;

    pub const FLOATS_PER_COLOR: usize = 4;
    pub const COLORS_PER_BUTTON: usize = 4;

    pub fn new(font_bytes: &'static [u8], window_size: (u32, u32), programs: [GLuint; 2]) -> Self {
		//Load font used for text rendering
		let font = match FontArc::try_from_slice(font_bytes) {
			Ok(s) => { s }
			Err(e) => { panic!("{}", e) }
		};

		let glyph_brush = GlyphBrushBuilder::using_font(font).build();

        //Create the glyph texture
        let glyph_texture = unsafe {
            let (width, height) = glyph_brush.texture_dimensions();
            let mut tex = 0;
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::GenTextures(1, &mut tex);
			gl::BindTexture(gl::TEXTURE_2D, tex);

			let params = [
				(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE),
				(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE),
				(gl::TEXTURE_MIN_FILTER, gl::NEAREST),
				(gl::TEXTURE_MAG_FILTER, gl::NEAREST),
			];
			glutil::apply_texture_parameters(&params);

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as GLint, width as GLint, height as GLint, 0, gl::RED, gl::UNSIGNED_BYTE, ptr::null());
            tex
        };

        UIState {
            button_color_buffer: 0,
            buttons_vao: None,
            internals: UIInternals::new(glyph_brush, window_size),
            glyph_texture,
            glyph_vao: None,
			glyph_count: 0,
			menu_chains: Vec::new(),
			menus: Vec::new(),
			text_elements: Vec::new(),
			programs,
			hidden_menus: None
        }
    }
	
	pub fn append_to_chain(&mut self, chain: usize, dst: usize) {
		//We only need to hide the current menu if there are more than zero menus in the chain
		if self.menu_chains[chain].len() > 0 {
			let src = self.menu_chains[chain][self.menu_chains[chain].len() - 1];
			self.hide_menu(src);
		}
		self.show_menu(dst);
		self.menu_chains[chain].push(dst);
	}

    pub fn button_count(&self) -> usize { self.internals.buttons.count() }

	pub fn create_menu_chain(&mut self) -> usize {
		self.menu_chains.push(Vec::new());
		self.menu_chains.len() - 1
	}

	pub unsafe fn draw(&self, screen_state: &render::ScreenState) {
		//Render UI buttons
		if let Some(vao) = self.buttons_vao {
			draw_ui_elements(vao, self.programs[Self::BUTTON_SHADER], self.button_count(), screen_state.get_clipping_from_screen());
		}

		//Render text
		if let Some(vao) = self.glyph_vao {
			gl::ActiveTexture(gl::TEXTURE0);
			gl::BindTexture(gl::TEXTURE_2D, self.glyph_texture);

			draw_ui_elements(vao, self.programs[Self::GLYPH_SHADER], self.glyph_count, screen_state.get_clipping_from_screen());
		}
	}

    fn hide_all_menus(&mut self) {
		let mut hid_menus = Vec::new();
        for i in 0..self.menus.len() {
            if self.menus[i].hide(&mut self.internals) {
				hid_menus.push(i);
			}
		}
		
		self.hidden_menus = Some(hid_menus);
	}
	
	fn unhide_all_menus(&mut self) {
		if let Some(menu_indices) = &self.hidden_menus {
			for index in menu_indices {
				self.menus[*index].show(&mut self.internals);
			}
			self.hidden_menus = None;
		}
	}

	pub fn toggle_hide_all_menus(&mut self) {
		if self.hidden_menus == None {
			self.hide_all_menus();
		} else {
			self.unhide_all_menus();
		}
	}

	fn hide_menu(&mut self, index: usize) { self.menus[index].hide(&mut self.internals); }

    //Clears the data in self.internals and marks all menus as inactive
    pub fn reset(&mut self) {
        self.internals.buttons.clear();
		self.internals.sections.clear();
		
		for text in self.text_elements.iter_mut() {
			text.active = false;
		}

        for menu in self.menus.iter_mut() {
			menu.active = false;
		}
		
		for chain in self.menu_chains.iter_mut() {
			chain.clear();
		}
    }
	
	pub fn resize(&mut self, new_screen_size: (u32, u32)) {
		self.internals.window_size = new_screen_size;
		for menu in self.menus.iter_mut() {
			if menu.active {
				menu.toggle(&mut self.internals);
				menu.toggle(&mut self.internals);
			}
		}

		for text_element in self.text_elements.iter_mut() {
			if text_element.active {
				text_element.toggle(&mut self.internals);
				text_element.toggle(&mut self.internals);
			}
		}
	}
	
	pub fn rollback_chain(&mut self, chain: usize) {
		if let Some(index) = self.menu_chains[chain].pop() {
			self.hide_menu(index);
			
			if self.menu_chains[chain].len() > 0 {
				let dst = self.menu_chains[chain][self.menu_chains[chain].len() - 1];
				self.show_menu(dst);
			}
		}
	}

	pub fn set_menus(&mut self, menus: Vec<Menu<'a, T>>) { self.menus = menus; }

	pub fn set_text_elements(&mut self, texts: Vec<UIText<'a>>) { self.text_elements = texts; }

	fn show_menu(&mut self, index: usize) { self.menus[index].show(&mut self.internals); }

	//Call this function each frame right before rendering
    pub fn synchronize(&mut self) {
		//Queue glyph_brush sections
		self.queue_sections();

		//glyph_brush processing
		self.glyph_processing();

		//Create vao for the ui buttons
		self.update_button_vao();
    }

    pub fn toggle_menu(&mut self, chain: usize, menu: usize) {
		if self.menus[menu].active {
			self.rollback_chain(chain);
		} else {
			self.append_to_chain(chain, menu);
		}
	}

	pub fn toggle_text_element(&mut self, index: usize) {
		self.text_elements[index].toggle(&mut self.internals);
	}

    //Gets input from the UI system and generates Commands for the command buffer I.E. user clicking on buttons
    //Also updates the instanced color buffer used for rendering the buttons
    //Meant to be called once per frame
    pub fn update_buttons(&mut self, screen_space_mouse: glm::TVec2<f32>, mouse_lbutton_pressed: bool, mouse_lbutton_pressed_last_frame: bool, command_buffer: &mut Vec<T>) {        
		//Handle input from the UI buttons
		let mut current_button = 0;
		for i in 0..self.internals.buttons.len() {
			if let Some(button) = self.internals.buttons.get_mut_element(i) {
				if screen_space_mouse.x > button.bounds.min[0] &&
				   screen_space_mouse.x < button.bounds.max[0] &&
				   screen_space_mouse.y > button.bounds.min[1] &&
				   screen_space_mouse.y < button.bounds.max[1] {

					if mouse_lbutton_pressed_last_frame && !mouse_lbutton_pressed {
						if let Some(command) = button.command {
							command_buffer.push(command);
						}
					}

					//Handle updating button graphics
					if button.state == ButtonState::None || (mouse_lbutton_pressed == mouse_lbutton_pressed_last_frame) {
						let color = if mouse_lbutton_pressed {
							[0.0, 0.8, 0.0, 0.5]
						} else {
							[0.0, 0.4, 0.0, 0.5]
						};
						unsafe { Self::update_ui_button_color(self.button_color_buffer, current_button, color); }

						button.state = ButtonState::Highlighted;
					}
				} else {
					if button.state != ButtonState::None {
						let color = [0.0, 0.0, 0.0, 0.5];
						unsafe { Self::update_ui_button_color(self.button_color_buffer, current_button, color); }

						button.state = ButtonState::None;
					}
				}				
				current_button += 1;
			}
		}
	}

    fn glyph_processing(&mut self) {
        let glyph_tex = self.glyph_texture;

        //glyph_brush processing
		let mut glyph_result = self.internals.glyph_brush.process_queued(|rect, tex_data| unsafe { 
			upload_glyph_texture(glyph_tex, rect, tex_data);
		}, glyph_vertex_transform);

		//Repeatedly resize the glyph texture until the error stops
		while let Err(BrushError::TextureTooSmall { suggested }) = glyph_result {
			let (width, height) = suggested;
			unsafe {
				gl::BindTexture(gl::TEXTURE_2D, self.glyph_texture);
				gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RED as GLint, width as GLint, height as GLint, 0, gl::RED, gl::UNSIGNED_BYTE, ptr::null());
			}
			self.internals.glyph_brush.resize_texture(width, height);
			glyph_result = self.internals.glyph_brush.process_queued(|rect, tex_data| unsafe {
				upload_glyph_texture(glyph_tex, rect, tex_data);
			}, glyph_vertex_transform);
		}
		
		//This should never fail
		match glyph_result.unwrap() {
			BrushAction::Draw(verts) => {
				if verts.len() > 0 {
					let mut vertex_buffer = Vec::with_capacity(verts.len() * FLOATS_PER_GLYPH);
					let mut index_buffer = vec![0; verts.len() * 6];
					for i in 0..verts.len() {
						for v in verts[i].iter() {
							vertex_buffer.push(*v);
						}
						
						//Fill out index buffer
						insert_index_buffer_quad(&mut index_buffer, i);
					}
					self.glyph_count = verts.len();

					let attribute_strides = [2, 2, 4];
					match self.glyph_vao {
						Some(mut vao) => unsafe {
							gl::DeleteVertexArrays(1, &mut vao);
							self.glyph_vao = Some(glutil::create_vertex_array_object(&vertex_buffer, &index_buffer, &attribute_strides));
						}
						None => unsafe {
							self.glyph_vao = Some(glutil::create_vertex_array_object(&vertex_buffer, &index_buffer, &attribute_strides));
						}
					}
				} else {
					if let Some(mut vao) = self.glyph_vao {
						unsafe { gl::DeleteVertexArrays(1, &mut vao); }
						self.glyph_vao = None;
					}
				}
			}
			BrushAction::ReDraw => {}
		}
    }

    fn queue_sections(&mut self) {
        for sec in self.internals.sections.iter() {
			if let Some(s) = sec {
				self.internals.glyph_brush.queue(s);
			}
		}
    }

    fn update_button_vao(&mut self) {
        //Create vao for the ui buttons
		if self.internals.vao_flag && self.button_count() > 0 {
			self.internals.vao_flag = false;
			unsafe { 
				let floats_per_button = 4 * 2;
				let mut vertices = vec![0.0; self.button_count() * floats_per_button];
				let mut indices = vec![0u16; self.button_count() * 6];

				let mut quads_added = 0;
				for i in 0..self.internals.buttons.len() {
					if let Some(button) = &self.internals.buttons[i] {
						vertices[quads_added * floats_per_button] = button.bounds.min[0];
						vertices[quads_added * floats_per_button + 1] = button.bounds.min[1];
						vertices[quads_added * floats_per_button + 2] = button.bounds.min[0];
						vertices[quads_added * floats_per_button + 3] = button.bounds.max[1];
						vertices[quads_added * floats_per_button + 4] = button.bounds.max[0];
						vertices[quads_added * floats_per_button + 5] = button.bounds.min[1];
						vertices[quads_added * floats_per_button + 6] = button.bounds.max[0];
						vertices[quads_added * floats_per_button + 7] = button.bounds.max[1];

						//Place this quad in the index buffer
						insert_index_buffer_quad(&mut indices, quads_added);
						quads_added += 1;
					}
				}

				match self.buttons_vao {
					Some(mut vao) => {
						gl::DeleteVertexArrays(1, &mut vao);
						self.buttons_vao = Some(glutil::create_vertex_array_object(&vertices, &indices, &[2]));
						gl::BindVertexArray(vao);
					}
					None => {
						let vao = glutil::create_vertex_array_object(&vertices, &indices, &[2]);
						self.buttons_vao = Some(vao);
						gl::BindVertexArray(vao);
					}
				}

				//Create GPU buffer for ui button colors
				self.button_color_buffer = {
					let element_count = self.button_count() * Self::COLORS_PER_BUTTON * Self::FLOATS_PER_COLOR;

					let mut data = vec![0.0f32; element_count];
					for i in 0..(data.len() / Self::FLOATS_PER_COLOR) {
						data[i * 4] = 0.0;
						data[i * 4 + 1] = 0.0;
						data[i * 4 + 2] = 0.0;
						data[i * 4 + 3] = 0.5;
					}

					let mut b = 0;
					gl::GenBuffers(1, &mut b);
					gl::BindBuffer(gl::ARRAY_BUFFER, b);
					gl::BufferData(gl::ARRAY_BUFFER, (element_count * mem::size_of::<GLfloat>()) as GLsizeiptr, &data[0] as *const f32 as *const c_void, gl::DYNAMIC_DRAW);

					//Attach buffer to vao
					gl::VertexAttribPointer(1,
											4,
											gl::FLOAT,
											gl::FALSE,
											(Self::FLOATS_PER_COLOR * mem::size_of::<GLfloat>()) as GLsizei,
											ptr::null());
					gl::EnableVertexAttribArray(1);

					b
				};
			}
		} else if self.button_count() == 0 {
			if let Some(mut vao) = self.buttons_vao {
				unsafe { gl::DeleteVertexArrays(1, &mut vao); }
				self.buttons_vao = None;
			}
		}
    }

    //Change the color of button at index to color
    unsafe fn update_ui_button_color(buffer: GLuint, index: usize, color: [f32; 4]) { //When color's size is Self::FLOATS_PER_COLOR it causes a compiler bug
        let mut data = vec![0.0; Self::FLOATS_PER_COLOR * Self::COLORS_PER_BUTTON];
        
        for i in 0..(data.len() / Self::FLOATS_PER_COLOR) {
            data[i * Self::FLOATS_PER_COLOR] = color[0];
            data[i * Self::FLOATS_PER_COLOR + 1] = color[1];
            data[i * Self::FLOATS_PER_COLOR + 2] = color[2];
            data[i * Self::FLOATS_PER_COLOR + 3] = color[3];
        }
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::BufferSubData(gl::ARRAY_BUFFER,
                        (Self::COLORS_PER_BUTTON * Self::FLOATS_PER_COLOR * index * mem::size_of::<GLfloat>()) as GLintptr,
                        (Self::FLOATS_PER_COLOR * Self::COLORS_PER_BUTTON * mem::size_of::<GLfloat>()) as GLsizeiptr,
                        &data[0] as *const GLfloat as *const c_void);
    }
}

pub struct UIText<'a> {
	text: &'a str,
	font_size: f32,
	color: [f32; 4],
	anchor: UIAnchor,
	active: bool,
	index: Option<usize>
}

impl<'a> UIText<'a> {
	pub fn new(text: &'a str, font_size: f32, anchor: UIAnchor) -> Self {
		UIText {
			text,
			font_size,
			color: [1.0, 1.0, 1.0, 1.0],
			anchor,
			active: false,
			index: None
		}
	}

	pub fn show<T>(&mut self, internals: &mut UIInternals<'a, T>) {
		if self.active { return; }
		self.active = true;

		//Create the section
		let the_section = {
			let section = Section::new();
			let mut text = Text::new(self.text).with_color(self.color);
			text.scale = PxScale::from(self.font_size);
			let mut section = section.add_text(text);
		
			let bounding_box = match internals.glyph_brush.glyph_bounds(&section) {
				Some(bb) => { bb }
				None => {
					panic!("Can't compute a bounding box for {}", self.text);
				}
			};

			section.screen_position = match self.anchor {
                UIAnchor::LeftAligned(pos) => { pos }
                UIAnchor::DeadCenter => {
					let x_pos = (internals.window_size.0 as f32 - bounding_box.width()) / 2.0;
					let y_pos = (internals.window_size.1 as f32 - bounding_box.height()) / 2.0;
					
					(x_pos, y_pos)
				}
				UIAnchor::CenterTop(offset) => {
					let x_pos = (internals.window_size.0 as f32 - bounding_box.width()) / 2.0;

					(x_pos, offset)
				}
			};
			
			section
		};

		self.index = Some(internals.sections.insert(the_section));
	}

	pub fn hide<T>(&mut self, internals: &mut UIInternals<T>) {
		if !self.active { return; }
		self.active = false;

		if let Some(i) = self.index {
			internals.delete_section(i);
		}
	}

	pub fn toggle<T>(&mut self, internals: &mut UIInternals<'a, T>) {
		if self.active {
			self.hide(internals);
		} else {
			self.show(internals);
		}
	}
}

//Free floating button element
#[derive(Debug)]
pub struct UIButton<T> {
    pub bounds: glyph_brush::Rectangle<f32>,
    pub state: ButtonState,
    pub command: Option<T>,
    section_id: usize
}

impl<T> UIButton<T> {
    pub fn new(section_id: usize, bounds: glyph_brush::Rectangle<f32>, command: Option<T>) -> Self {
        UIButton {
            bounds,
            state: ButtonState::None,
            command,
            section_id
        }
    }

    pub fn section_id(&self) -> usize { self.section_id }
}

pub struct Menu<'a, T> {
	button_labels: Vec<&'a str>,
	button_commands: Vec<Option<T>>,
	label_colors: Vec<[f32; 4]>,
    anchor: UIAnchor,
	active: bool,
	font_size: f32,
    ids: Vec<usize> //Indices into the buttons OptionVec. These are only valid when self.active == true
}

impl<'a, T: Copy> Menu<'a, T> {
    pub fn new(buttons: Vec<(&'a str, Option<T>)>, anchor: UIAnchor, font_size: f32) -> Self {
		let size = buttons.len();
		let mut button_labels = Vec::with_capacity(size);
		let mut button_commands = Vec::with_capacity(size);

		//lol
		for butt in buttons.iter() {
			button_labels.push(butt.0);
			button_commands.push(butt.1);
		}

		let label_colors = vec![[1.0, 1.0, 1.0, 1.0]; size];
		
        Menu {
            button_labels,
            button_commands,
            label_colors,
            anchor,
			active: false,
			font_size,
            ids: vec![0; size]
        }
	}
	
	pub fn new_with_colors(buttons: Vec<(&'a str, Option<T>, [f32; 4])>, anchor: UIAnchor, font_size: f32) -> Self {
		let size = buttons.len();
		let mut button_labels = Vec::with_capacity(size);
		let mut button_commands = Vec::with_capacity(size);
		let mut label_colors = Vec::with_capacity(size);
		for butt in buttons.iter() {
			button_labels.push(butt.0);
			button_commands.push(butt.1);
			label_colors.push(butt.2);
		}

        Menu {
            button_labels,
            button_commands,
            label_colors,
            anchor,
			active: false,
			font_size,
            ids: vec![0; size]
        }
	}

    //Adds this menu's data to the arrays of buttons and sections
    pub fn show(&mut self, ui_internals: &mut UIInternals<'a, T>) {
        if self.active { return; }

        //Submit the pause menu data
		const BORDER_PROPORTION: f32 = 0.2;
		const BUFFER_DISTANCE: f32 = 10.0;
		for i in 0..self.button_labels.len() {
			let mut section = {
				let section = Section::new();
				let mut text = Text::new(self.button_labels[i]).with_color(self.label_colors[i]);
				text.scale = PxScale::from(self.font_size);
				section.add_text(text)
			};
			let bounding_box = match ui_internals.glyph_brush.glyph_bounds(&section) {
				Some(rect) => { rect }
				None => { continue; }
			};

			//Create the associated UI button
			let border = bounding_box.height() * BORDER_PROPORTION;
			let width = bounding_box.width() + 2.0 * border;
			let height = bounding_box.height() + 2.0 * border;

            let button_bounds = match self.anchor {
                UIAnchor::LeftAligned((x, y)) => {
                    let x_pos = x;
                    let y_pos = y + i as f32 * (height + BUFFER_DISTANCE);
                    glyph_brush::Rectangle {
                        min: [x_pos, y_pos],
                        max: [x_pos + width, y_pos + height]
                    }
                }
                UIAnchor::DeadCenter => {
					let total_menu_height = (height + BUFFER_DISTANCE) * self.button_labels.len() as f32 - BUFFER_DISTANCE;

					let x_pos = (ui_internals.window_size.0 as f32 - width) / 2.0;
					let y_pos = (ui_internals.window_size.1 as f32 - total_menu_height) / 2.0 + i as f32 * (height + BUFFER_DISTANCE);
                    glyph_brush::Rectangle {
                        min: [x_pos, y_pos],
                        max: [x_pos + width, y_pos + height]
                    }
				}
				UIAnchor::CenterTop(offset) => {
					let x_pos = (ui_internals.window_size.0 as f32 - width) / 2.0;
					let y_pos = offset + i as f32 * (height + BUFFER_DISTANCE);
                    glyph_brush::Rectangle {
                        min: [x_pos, y_pos],
                        max: [x_pos + width, y_pos + height]
                    }
				}
            };
					
		    section.screen_position = (
			    button_bounds.min[0] + border,
			    button_bounds.min[1] + border
		    );

		    //Finally insert the section into the array
		    let section_id = ui_internals.sections.insert(section);

    		let button = UIButton::new(section_id, button_bounds, self.button_commands[i]);
    		self.ids[i] = ui_internals.add_button(button);
        }
		self.active = true;
    }

	//Remove this menu's data from the arrays of buttons and sections
	//Returns true if the menu was actually visible
    pub fn hide(&mut self, ui_internals: &mut UIInternals<'a, T>) -> bool {
        if !self.active { return false; }
		for id in self.ids.iter() {
			ui_internals.delete_button(*id);
        }
		self.active = false;
		return true;
    }

    pub fn toggle(&mut self, ui_internals: &mut UIInternals<'a, T>) {
        if self.active {
            self.hide(ui_internals);
        } else {
            self.show(ui_internals);
        }
    }
}

//Defines the anchor point of the UI element and how that anchor is configured
pub enum UIAnchor {
    LeftAligned((f32, f32)),			//Parameter is the screen-space position of the top-left corner of the entire menu's bounding box
	DeadCenter,
	CenterTop(f32)						//Parameter is the offset from the top in pixels
}

#[derive(PartialEq, Eq, Debug)]
pub enum ButtonState {
    None,
    Highlighted
}