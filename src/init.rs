use glfw::{Glfw, Window, WindowEvent, WindowMode};
use std::sync::mpsc::Receiver;

pub fn glfw_window(window_size: (u32, u32), window_mode: WindowMode, major_version: u32, minor_version: u32, title: &str) -> (Glfw, Window, Receiver<(f64, WindowEvent)>) {
	//Init glfw
	let mut glfw = match glfw::init(glfw::FAIL_ON_ERRORS) {
		Ok(g) => { g }
		Err(e) => {	panic!("GLFW init error: {}", e); }
	};

	glfw.window_hint(glfw::WindowHint::ContextVersion(major_version, minor_version));
	glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

	//Create window
    let (window, events) = glfw.create_window(window_size.0, window_size.1, title, window_mode).unwrap();
    (glfw, window, events)
}