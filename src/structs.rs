use gl::types::*;
use std::slice::{Iter, IterMut};
use std::ops::{Index, IndexMut};
use wavefront_obj::mtl;

#[derive(Debug)]
pub struct VertexArray {
	pub vertices: Vec<f32>,
	pub indices: Vec<u16>,
	pub attribute_offsets: Vec<i32>
}

//The way MeshData works is that vertex_array can hold any number of meshes, and then
//the names, geo_boundaries, and materials Vecs are all parallel
pub struct MeshData {
	pub vertex_array: VertexArray,
	pub names: Vec<String>,
	pub geo_boundaries: Vec<GLsizei>,			//[0, a, b, c, ..., indices.length - 1]
	pub materials: Vec<mtl::Material>
}

pub struct OzyMesh {
	pub vertex_array: VertexArray,
	pub names: Vec<String>,
	pub geo_boundaries: Vec<u16>,			//[0, a, b, c, ..., indices.length - 1]
}

pub struct ImageData {
	pub data: Vec<u8>,
	pub width: i32,
	pub height: i32,
	pub format: GLenum,
	pub internal_format: GLenum
}

//A wrapper for the useful Vec<Option<T>> pattern
pub struct OptionVec<T> {
	optionvec: Vec<Option<T>>
}

impl<T> OptionVec<T> {
	pub fn new() -> Self {
		OptionVec {
			optionvec: Vec::new()
		}
	}

	pub fn with_capacity(size: usize) -> Self {
		OptionVec {
			optionvec: Vec::with_capacity(size)
		}
	}

	pub fn insert(&mut self, element: T) -> usize {
		let mut index = None;

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

	pub fn clear(&mut self) {
		for i in 0..self.optionvec.len() {
			self.optionvec[i] = None;
		}
	}

	pub fn len(&self) -> usize {
		self.optionvec.len()
	}

	pub fn get_element(&mut self, index: Option<usize>) -> Option<&mut T> {	
		match index {
			Some(i) => {
				self[i].as_mut()
			}
			None => { None }
		}	
	}

	pub fn two_mut_refs(&mut self, index1: usize, index2: usize) -> (&mut Option<T>, &mut Option<T>) {
		//Deternime which index is larger
		if index1 < index2 {
			let (first, second) = self.optionvec.split_at_mut(index1 + 1);
			let first_len = first.len();

			(&mut first[first_len - 1], &mut second[index2 - index1 - 1])
		} else {			
			let (first, second) = self.optionvec.split_at_mut(index2 + 1);
			let first_len = first.len();
			
			(&mut second[index1 - index2 - 1], &mut first[first_len - 1])
		}
	}

	pub fn iter(&self) -> Iter<Option<T>> {
		self.optionvec.iter()
	}

	pub fn _iter_mut(&mut self) -> IterMut<Option<T>> {
		self.optionvec.iter_mut()
	}
}

impl<T> Index<usize> for OptionVec<T> {
	type Output = Option<T>;

	fn index(&self, index: usize) -> &Self::Output {
		&self.optionvec[index]
	}
}

impl<T> IndexMut<usize> for OptionVec<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.optionvec[index]
	}
}