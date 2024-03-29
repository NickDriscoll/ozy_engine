use gl::types::*;
use std::slice::{Iter, IterMut};
use std::ops::{Index};
use std::time::{Instant};

//Struct for basic frame timing ops
pub struct FrameTimer {
	pub frame_count: u64,
	pub last_frame_instant: Instant,
	pub elapsed_time: f32,
	pub delta_time: f32
}

impl FrameTimer {
	const MAX_TIME: f32 = 60.0;

	pub fn new() -> Self {
		FrameTimer {
			frame_count: 0,
			last_frame_instant: Instant::now(),
			elapsed_time: 0.0,
			delta_time: 0.0
		}
	}

	pub fn update(&mut self) {
		self.delta_time = Instant::now().duration_since(self.last_frame_instant).as_secs_f32();
		self.elapsed_time += self.delta_time;
		self.last_frame_instant = Instant::now();
		self.frame_count += 1;
	}
}

#[derive(Debug)]
pub struct VertexArray {
	pub vertices: Vec<f32>,
	pub indices: Vec<u32>,
	pub attribute_offsets: Vec<i32>
}

pub struct UninterleavedVertexArrays {
    pub positions: Vec<f32>,
    pub tangents: Vec<f32>,
    pub normals: Vec<f32>,
    pub uvs: Vec<f32>,
}

pub struct ImageData {
	pub data: Vec<u8>,
	pub width: i32,
	pub height: i32,
	pub format: GLenum,
	pub internal_format: GLenum
}

//A wrapper for the useful Vec<Option<T>> pattern
#[derive(Debug)]
pub struct OptionVec<T> {
	optionvec: Vec<Option<T>>,
	count: usize		//True number of active objects
}

impl<T> OptionVec<T> {
	pub fn new() -> Self {
		OptionVec {
			optionvec: Vec::new(),
			count: 0
		}
	}

	pub fn as_slice(&self) -> &[Option<T>] {
		self.optionvec.as_slice()
	}

	pub fn with_capacity(size: usize) -> Self {
		OptionVec {
			optionvec: Vec::with_capacity(size),
			count: 0
		}
	}

	pub fn insert(&mut self, element: T) -> usize {
		let mut index = None;

		//Increment count
		self.count += 1;

		//Search for an empty space
		for i in 0..self.optionvec.len() {
			if let None = self.optionvec[i] {
				index = Some(i);
				break;
			}
		}

		//Fill the empty space if one was found, push onto the end otherwise
		match index {
			Some(i) => {
				self.optionvec[i] = Some(element);
				i
			}
			None => {
				self.optionvec.push(Some(element));
				self.optionvec.len() - 1
			}
		}
	}

	pub fn replace(&mut self, index: usize, item: T) {
		if let Some(_) = self.optionvec[index] {
			self.optionvec[index] = Some(item);
		}
	} 

	pub fn delete(&mut self, index: usize) -> Option<T> {
		let mut res = None;
		if let Some(_) = self.optionvec[index] {
			self.count -= 1;
			std::mem::swap(&mut res, self.optionvec.get_mut(index).unwrap());
		}
		res
	}

	pub fn clear(&mut self) {
		self.optionvec.clear();
		self.count = 0;
	}

	pub fn len(&self) -> usize { self.optionvec.len() }

	pub fn count(&self) -> usize { self.count }

	pub fn get_element(&mut self, index: usize) -> Option<&T> {
		self.optionvec[index].as_ref()
	}

	pub fn get_mut_element(&mut self, index: usize) -> Option<&mut T> {
		self.optionvec[index].as_mut()
	}

	pub fn iter(&self) -> Iter<Option<T>> {
		self.optionvec.iter()
	}

	pub fn iter_mut(&mut self) -> IterMut<Option<T>> {
		self.optionvec.iter_mut()
	}
}

impl<T> Index<usize> for OptionVec<T> {
	type Output = Option<T>;

	fn index(&self, index: usize) -> &Self::Output {
		&self.optionvec[index]
	}
}