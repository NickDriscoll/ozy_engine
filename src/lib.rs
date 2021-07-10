#![allow(dead_code)]
extern crate nalgebra_glm as glm;

pub mod collision;
pub mod init;
pub mod io;
pub mod structs;
pub mod glutil;
pub mod prims;
pub mod render;
pub mod routines;
pub mod ui;

fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min { min }
    else if x > max { max }
    else { x }
}