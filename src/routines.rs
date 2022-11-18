
#[inline]
pub fn calculate_mipcount(width: u32, height: u32) -> u32 {
    (f32::floor(f32::log2(u32::max(width, height) as f32))) as u32 + 1
}

pub fn mip_resolution(width: u32, height: u32, level: u32) -> (u32, u32) {
    let w = u32::max(width >> level, 1);
    let h = u32::max(height >> level, 1);
    (w, h)
}

pub fn uniform_scale(scale: f32) -> glm::TMat4<f32> {
	glm::scaling(&glm::vec3(scale, scale, scale))
}
