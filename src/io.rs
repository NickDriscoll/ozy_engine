#![allow(non_camel_case_types)]
use std::mem::{self, size_of};
use std::fs::File;
use std::io::{Error, Read, Result, Write};
use std::path::Path;
use std::string::String;
use crate::{structs::*, routines, render::PositionNormalTangentUvPrimitive};

//DDS Programming Guide
//https://learn.microsoft.com/en-us/windows/win32/direct3ddds/dx-graphics-dds-pguide

/*
struct DDS_PIXELFORMAT {
  DWORD dwSize;
  DWORD dwFlags;
  DWORD dwFourCC;
  DWORD dwRGBBitCount;
  DWORD dwRBitMask;
  DWORD dwGBitMask;
  DWORD dwBBitMask;
  DWORD dwABitMask;
};
*/
#[derive(Debug)]
#[repr(C)]
pub struct DDS_PixelFormat {
    pub size: u32,
    pub flags: u32,
    pub four_cc: u32,
    pub rgb_bitcount: u32,
    pub r_bitmask: u32,
    pub g_bitmask: u32,
    pub b_bitmask: u32,
    pub a_bitmask: u32,
}

impl DDS_PixelFormat {
    pub const DDPF_ALPHAPIXELS: u32 = 0x1;
    pub const DDPF_ALPHA: u32 = 0x2;
    pub const DDPF_FOURCC: u32 = 0x4;
    pub const DDPF_RGB: u32 = 0x1;
    pub const DDPF_YUV: u32 = 0x2;
    pub const DDPF_LUMINANCE: u32 = 0x4;

    pub fn from_header_bytes(header_bytes: &[u8]) -> Self {
        let start = 19 * size_of::<u32>();
        let mut current_offset = 0;

        let size = read_u32_from_le_bytes(header_bytes, start + current_offset);
        current_offset += size_of::<u32>();

        let flags = read_u32_from_le_bytes(header_bytes, start + current_offset);
        current_offset += size_of::<u32>();

        let four_cc = read_u32_from_le_bytes(header_bytes, start + current_offset);
        current_offset += size_of::<u32>();

        let rgb_bitcount = read_u32_from_le_bytes(header_bytes, start + current_offset);
        current_offset += size_of::<u32>();

        let r_bitmask = read_u32_from_le_bytes(header_bytes, start + current_offset);
        current_offset += size_of::<u32>();

        let g_bitmask = read_u32_from_le_bytes(header_bytes, start + current_offset);
        current_offset += size_of::<u32>();

        let b_bitmask = read_u32_from_le_bytes(header_bytes, start + current_offset);
        current_offset += size_of::<u32>();

        let a_bitmask = read_u32_from_le_bytes(header_bytes, start + current_offset);
        

        DDS_PixelFormat {
            size,
            flags,
            four_cc,
            rgb_bitcount,
            r_bitmask,
            g_bitmask,
            b_bitmask,
            a_bitmask
        }
    }
}

impl Default for DDS_PixelFormat {
    fn default() -> Self {
        let mut four_cc = 0u32;
        four_cc |= ('D' as u32) << 0;
        four_cc |= ('X' as u32) << 8;
        four_cc |= ('1' as u32) << 16;
        four_cc |= ('0' as u32) << 24;
        DDS_PixelFormat {
            size: 32,
            flags: 0,
            four_cc,
            rgb_bitcount: 8,
            r_bitmask: 0,
            g_bitmask: 0,
            b_bitmask: 0,
            a_bitmask: 0,
        }
    }
}

/*
typedef struct {
     DWORD           dwSize;
     DWORD           dwFlags;
     DWORD           dwHeight;
     DWORD           dwWidth;
     DWORD           dwPitchOrLinearSize;
     DWORD           dwDepth;
     DWORD           dwMipMapCount;
     DWORD           dwReserved1[11];
     DDS_PIXELFORMAT ddspf;
     DWORD           dwCaps;
     DWORD           dwCaps2;
     DWORD           dwCaps3;
     DWORD           dwCaps4;
     DWORD           dwReserved2;
} DDS_HEADER;
   */
#[derive(Debug)]
#[repr(C)]
pub struct DDSHeader {
    pub magic_word: u32,        // 0x20534444
    pub size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mipmap_count: u32,
    pub reserved_1: [u32; 11],
    pub spf: DDS_PixelFormat,
    pub caps: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub caps4: u32,
    pub reserved2: u32,
    pub dx10_header: DDSHeader_DXT10
}

impl DDSHeader {
    const TRUE_BC7_HEADER_SIZE: usize = 148;        //This is DDSHeader + DDSHeader_DXT10 + magic word
    const REQUIRED_BC7_SIZE: u32 = 124;

    pub const DDSD_CAPS: u32 = 0x1;
    pub const DDSD_HEIGHT: u32 = 0x2;
    pub const DDSD_WIDTH: u32 = 0x4;
    pub const DDSD_PITCH: u32 = 0x8;
    pub const DDSD_PIXELFORMAT: u32 = 0x1000;
    pub const DDSD_MIPMAPCOUNT: u32 = 0x20000;
    pub const DDSD_LINEARSIZE: u32 = 0x80000;
    pub const DDSD_DEPTH: u32 = 0x800000;

    pub fn from_file(dds_file: &mut File) -> Self {
        let mut header_buffer = vec![0u8; Self::TRUE_BC7_HEADER_SIZE];
    
        dds_file.read_exact(&mut header_buffer).unwrap();
    
        let height = read_u32_from_le_bytes(&header_buffer, 12);
        let width = read_u32_from_le_bytes(&header_buffer, 16);
        let pitch_or_linear_size = read_u32_from_le_bytes(&header_buffer, 20);
        let mipmap_count = read_u32_from_le_bytes(&header_buffer, 28);
        let pixel_format = DDS_PixelFormat::from_header_bytes(&header_buffer);

        let dx10_header = DDSHeader_DXT10::from_header_bytes(&header_buffer);

        DDSHeader {
            height,
            width,
            pitch_or_linear_size,
            mipmap_count,
            spf: pixel_format,
            dx10_header,
            ..Default::default()
        }
    }
}

impl Default for DDSHeader {
    fn default() -> Self {
        DDSHeader {
            magic_word: 0x20534444,
            size: Self::REQUIRED_BC7_SIZE,
            flags: 0,
            height: 0,
            width: 0,
            pitch_or_linear_size: 0,
            depth: 0,
            mipmap_count: 1,
            reserved_1: [0; 11],
            spf: DDS_PixelFormat::default(),
            caps: 0,
            caps2: 0,
            caps3: 0,
            caps4: 0,
            reserved2: 0,
            dx10_header: DDSHeader_DXT10 {
                dxgi_format: DXGI_FORMAT::UNKNOWN,
                resource_dimension: D3D10_RESOURCE_DIMENSION::TEXTURE2D,
                misc_flag: 0,
                array_size: 1,
                misc_flags2: 0
            }
        }
    }
}

/*
typedef struct {
  DXGI_FORMAT              dxgiFormat;
  D3D10_RESOURCE_DIMENSION resourceDimension;
  UINT                     miscFlag;
  UINT                     arraySize;
  UINT                     miscFlags2;
} DDS_HEADER_DXT10;
 */
#[derive(Debug)]
#[repr(C)]
pub struct DDSHeader_DXT10 {
    pub dxgi_format: DXGI_FORMAT,
    pub resource_dimension: D3D10_RESOURCE_DIMENSION,
    pub misc_flag: u32,
    pub array_size: u32,
    pub misc_flags2: u32
}

impl DDSHeader_DXT10 {
    pub fn from_header_bytes(header_bytes: &[u8]) -> Self {
        let start = size_of::<DDSHeader>() - size_of::<DDSHeader_DXT10>();
        let mut current_offset = 0;

        let dxgi_format = read_u32_from_le_bytes(header_bytes, start + current_offset);
        let dxgi_format = unsafe { std::mem::transmute(dxgi_format) };
        current_offset += size_of::<u32>();

        let resource_dimension = read_u32_from_le_bytes(header_bytes, start + current_offset);
        let resource_dimension = unsafe { std::mem::transmute(resource_dimension) };
        current_offset += size_of::<u32>();

        let misc_flag = read_u32_from_le_bytes(header_bytes, start + current_offset);
        current_offset += size_of::<u32>();

        let array_size = read_u32_from_le_bytes(header_bytes, start + current_offset);
        current_offset += size_of::<u32>();

        let misc_flags2 = read_u32_from_le_bytes(header_bytes, start + current_offset);
        
        DDSHeader_DXT10 {
            dxgi_format,
            resource_dimension,
            misc_flag,
            array_size,
            misc_flags2
        }
    }
}

impl Default for DDSHeader_DXT10 {
    fn default() -> Self {
        DDSHeader_DXT10 {
            dxgi_format: DXGI_FORMAT::UNKNOWN,
            resource_dimension: D3D10_RESOURCE_DIMENSION::TEXTURE2D,
            misc_flag: 0,
            array_size: 1,
            misc_flags2: 0
        }
    }
}

impl DDSHeader_DXT10 {
    pub const DDS_RESOURCE_MISC_TEXTURECUBE: u32 = 0x4;
}

#[derive(Debug)]
pub enum D3D10_RESOURCE_DIMENSION {
    UNKNOWN = 0,
    BUFFER = 1,
    TEXTURE1D = 2,
    TEXTURE2D = 3,
    TEXTURE3D = 4,

    _RESERVED = 0xFFFFFFFF
}

#[derive(Debug)]
pub enum DXGI_FORMAT {
    UNKNOWN = 0,
    R32G32B32A32_TYPELESS = 1,
    R32G32B32A32_FLOAT = 2,
    R32G32B32A32_UINT = 3,
    R32G32B32A32_SINT = 4,
    R32G32B32_TYPELESS = 5,
    R32G32B32_FLOAT = 6,
    R32G32B32_UINT = 7,
    R32G32B32_SINT = 8,
    R16G16B16A16_TYPELESS = 9,
    R16G16B16A16_FLOAT = 10,
    R16G16B16A16_UNORM = 11,
    R16G16B16A16_UINT = 12,
    R16G16B16A16_SNORM = 13,
    R16G16B16A16_SINT = 14,
    R32G32_TYPELESS = 15,
    R32G32_FLOAT = 16,
    R32G32_UINT = 17,
    R32G32_SINT = 18,
    R32G8X24_TYPELESS = 19,
    D32_FLOAT_S8X24_UINT = 20,
    R32_FLOAT_X8X24_TYPELESS = 21,
    X32_TYPELESS_G8X24_UINT = 22,
    R10G10B10A2_TYPELESS = 23,
    R10G10B10A2_UNORM = 24,
    R10G10B10A2_UINT = 25,
    R11G11B10_FLOAT = 26,
    R8G8B8A8_TYPELESS = 27,
    R8G8B8A8_UNORM = 28,
    R8G8B8A8_UNORM_SRGB = 29,
    R8G8B8A8_UINT = 30,
    R8G8B8A8_SNORM = 31,
    R8G8B8A8_SINT = 32,
    R16G16_TYPELESS = 33,
    R16G16_FLOAT = 34,
    R16G16_UNORM = 35,
    R16G16_UINT = 36,
    R16G16_SNORM = 37,
    R16G16_SINT = 38,
    R32_TYPELESS = 39,
    D32_FLOAT = 40,
    R32_FLOAT = 41,
    R32_UINT = 42,
    R32_SINT = 43,
    R24G8_TYPELESS = 44,
    D24_UNORM_S8_UINT = 45,
    R24_UNORM_X8_TYPELESS = 46,
    X24_TYPELESS_G8_UINT = 47,
    R8G8_TYPELESS = 48,
    R8G8_UNORM = 49,
    R8G8_UINT = 50,
    R8G8_SNORM = 51,
    R8G8_SINT = 52,
    R16_TYPELESS = 53,
    R16_FLOAT = 54,
    D16_UNORM = 55,
    R16_UNORM = 56,
    R16_UINT = 57,
    R16_SNORM = 58,
    R16_SINT = 59,
    R8_TYPELESS = 60,
    R8_UNORM = 61,
    R8_UINT = 62,
    R8_SNORM = 63,
    R8_SINT = 64,
    A8_UNORM = 65,
    R1_UNORM = 66,
    R9G9B9E5_SHAREDEXP = 67,
    R8G8_B8G8_UNORM = 68,
    G8R8_G8B8_UNORM = 69,
    BC1_TYPELESS = 70,
    BC1_UNORM = 71,
    BC1_UNORM_SRGB = 72,
    BC2_TYPELESS = 73,
    BC2_UNORM = 74,
    BC2_UNORM_SRGB = 75,
    BC3_TYPELESS = 76,
    BC3_UNORM = 77,
    BC3_UNORM_SRGB = 78,
    BC4_TYPELESS = 79,
    BC4_UNORM = 80,
    BC4_SNORM = 81,
    BC5_TYPELESS = 82,
    BC5_UNORM = 83,
    BC5_SNORM = 84,
    B5G6R5_UNORM = 85,
    B5G5R5A1_UNORM = 86,
    B8G8R8A8_UNORM = 87,
    B8G8R8X8_UNORM = 88,
    R10G10B10_XR_BIAS_A2_UNORM = 89,
    B8G8R8A8_TYPELESS = 90,
    B8G8R8A8_UNORM_SRGB = 91,
    B8G8R8X8_TYPELESS = 92,
    B8G8R8X8_UNORM_SRGB = 93,
    BC6H_TYPELESS = 94,
    BC6H_UF16 = 95,
    BC6H_SF16 = 96,
    BC7_TYPELESS = 97,
    BC7_UNORM = 98,
    BC7_UNORM_SRGB = 99,
    AYUV = 100,
    Y410 = 101,
    Y416 = 102,
    NV12 = 103,
    P010 = 104,
    P016 = 105,
    //420_OPAQUE = 106,
    YUY2 = 107,
    Y210 = 108,
    Y216 = 109,
    NV11 = 110,
    AI44 = 111,
    IA44 = 112,
    P8 = 113,
    A8P8 = 114,
    B4G4R4A4_UNORM = 115,
    P208 = 130,
    V208 = 131,
    V408 = 132,
    SAMPLER_FEEDBACK_MIN_MIP_OPAQUE,
    SAMPLER_FEEDBACK_MIP_REGION_USED_OPAQUE,
    FORCE_UINT = 0xffffffff
}

pub fn compute_pitch_bc(width: u32, block_size: u32) -> u32 {
    //max( 1, ((width+3)/4) ) * block-size
    u32::max(1, (width + 3) / 4) * block_size
}

pub enum IndexType {
    U16,
    U32
}

#[derive(Clone)]
pub struct OzyMaterial {
    pub base_color: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub base_roughness: f32,
    pub color_bc7_idx: Option<u32>,
    pub normal_bc7_idx: Option<u32>,
    pub arm_bc7_idx: Option<u32>,          //arm == ambient(R), roughness(G), metallic(B)
    pub emissive_bc7_idx: Option<u32>,
}

impl Default for OzyMaterial {
    fn default() -> Self {
        OzyMaterial {
            base_color: [69.0; 4],
            emissive_factor: [0.0; 3],
            base_roughness: 0.0,
            color_bc7_idx: None,
            normal_bc7_idx: None,
            arm_bc7_idx: None,
            emissive_bc7_idx: None
        }
    }
}

#[derive(Clone, Default)]
pub struct OzyImage {
    pub width: u32,
    pub height: u32,
    pub mipmap_count: u32,
    pub bc7_bytes: Vec<u8>
}

pub struct OzyPrimitive {
    pub indices: Vec<u32>,
    pub vertex_positions: Vec<f32>,
    pub vertex_normals: Vec<f32>,
    pub vertex_tangents: Vec<f32>,
    pub vertex_uvs: Vec<f32>,
    pub material_idx: u32
}

impl PositionNormalTangentUvPrimitive for OzyPrimitive {
    fn vertex_positions(&self) -> &[f32] {
        &self.vertex_positions
    }

    fn vertex_normals(&self) -> &[f32] {
        &self.vertex_normals
    }

    fn vertex_tangents(&self) -> &[f32] {
        &self.vertex_tangents
    }

    fn vertex_uvs(&self) -> &[f32] {
        &self.vertex_uvs
    }
}

pub struct OzyMesh {
    pub name: String,
    pub textures: Vec<OzyImage>,
    pub materials: Vec<OzyMaterial>,
    pub primitives: Vec<OzyPrimitive>
}

impl OzyMesh {
    pub fn from_file(path: &str) -> Self {
        let name = String::from(Path::new(&path).file_stem().unwrap().to_str().unwrap());

        let mut file = File::open(path).unwrap();
        
        //Read header
        let material_count = read_u32(&mut file).unwrap();
        let primitive_count = read_u32(&mut file).unwrap();
        let texture_count = read_u32(&mut file).unwrap();

        let mut materials = Vec::with_capacity(material_count as usize);
        let mut primitives = Vec::with_capacity(primitive_count as usize);
        let mut textures = Vec::with_capacity(texture_count as usize);

        for _ in 0..material_count {
            let base_color = {
                let b = read_f32_data(&mut file, 4).unwrap();
                let mut o = [0.0; 4];
                for j in 0..4 {
                    o[j] = b[j];
                }
                o
            };
            let emissive_factor = {
                let b = read_f32_data(&mut file, 3).unwrap();
                let mut o = [0.0; 3];
                for j in 0..3 {
                    o[j] = b[j];
                }
                o
            };
            let base_roughness = read_f32(&mut file).unwrap();

            let idx = read_u32(&mut file).unwrap();
            let color_bc7_idx = if idx == 0xFFFFFFFF {
                None
            } else {
                Some(idx)
            };

            let idx = read_u32(&mut file).unwrap();
            let normal_bc7_idx = if idx == 0xFFFFFFFF {
                None
            } else {
                Some(idx)
            };

            let idx = read_u32(&mut file).unwrap();
            let arm_bc7_idx = if idx == 0xFFFFFFFF {
                None
            } else {
                Some(idx)
            };

            let idx = read_u32(&mut file).unwrap();
            let emissive_bc7_idx = if idx == 0xFFFFFFFF {
                None
            } else {
                Some(idx)
            };

            let mat = OzyMaterial {
                base_color,
                emissive_factor,
                base_roughness,
                color_bc7_idx,
                normal_bc7_idx,
                arm_bc7_idx,
                emissive_bc7_idx
            };
            materials.push(mat);
        }

        for _ in 0..primitive_count {
            let material_idx = read_u32(&mut file).unwrap();
            let indices = read_u32_array(&mut file).unwrap();
            let vertex_positions = read_f32_array(&mut file).unwrap();
            let vertex_normals = read_f32_array(&mut file).unwrap();
            let vertex_tangents = read_f32_array(&mut file).unwrap();
            let vertex_uvs = read_f32_array(&mut file).unwrap();

            let prim = OzyPrimitive {
                material_idx,
                indices,
                vertex_positions,
                vertex_normals,
                vertex_tangents,
                vertex_uvs
            };
            primitives.push(prim);
        }

        for _ in 0..texture_count {
            let width = read_u32(&mut file).unwrap();
            let height = read_u32(&mut file).unwrap();
            //let mipmap_count = routines::calculate_miplevels(width, height);
            let mipmap_count = read_u32(&mut file).unwrap();
            
            let mut bc7_byte_count = 0;
            for i in 0..mipmap_count {
                let (w, h) = routines::mip_resolution(width, height, i);
                bc7_byte_count += ispc::bc7::calc_output_size(w, h);
            }
            let bc7_bytes = read_u8_data(&mut file, bc7_byte_count).unwrap();

            let image = OzyImage {
                width,
                height,
                mipmap_count,
                bc7_bytes
            };
            textures.push(image);
        }

        OzyMesh {
            name,
            materials,
            primitives,
            textures
        }
    }
}

#[derive(Debug)]
pub struct OzyMeshOBSOLETE {
	pub vertex_array: VertexArray,
	pub texture_name: String,
    pub uv_velocity: [f32; 2],
    pub colors: Vec<f32>,
    pub is_transparent: bool
}

impl OzyMeshOBSOLETE {
    pub fn load(path: &str) -> Option<Self> {
        let mut texture_name = String::new();
        let mut colors = vec![];

        //Open the file
        let mut model_file = match File::open(path) {
            Ok(file) => { file }
            Err(e) => {
                println!("Unable to open \"{}\": {}", path, e);
                return None;
            }
        };

        //Check how many solid colors there are
        //If 0, it means this model uses textures
        let color_count = match read_u8(&mut model_file) {
            Ok(count) => { count as usize }
            Err(e) => {
                println!("{}", e);
                return None;
            }
        };

        //Branching on whether or not the model is textured or uses solid colors
        if color_count == 0 {
            //Read the material name
            texture_name = match read_pascal_strings(&mut model_file, 1) {
                Ok(v) => { v[0].clone() }
                Err(_) => { return None; }
            };
        } else {
            //Read the color f32s into a Vec
            colors = match read_f32_data(&mut model_file, color_count * 4) {
                Ok(data_block) => { data_block }
                Err(e) => {
                    println!("Error reading color data: {}", e);
                    return None;
                }
            };
        }

        //Transparency flag
        let is_transparent = match read_u8(&mut model_file) {
            Ok(flag) => { flag != 0 }
            Err(e) => {
                println!("Error reading transparency flag: {}", e);
                return None;
            }
        };

        //The uv_velocity
        let uv_velocity = match read_f32_data(&mut model_file, 2) {
            Ok(data) => { [data[0], data[1]] }
            Err(e) => { 
                println!("Error reading uv_velocity: {}", e);
                return None;
            }
        };
    
        //The length of the vertex data section of the file, in bytes
        let vertices_size = match read_u32(&mut model_file) {
            Ok(n) => { n }
            Err(_) => { return None; }
        };
    
        let vertices = {
            let mut bytes = vec![0; vertices_size as usize];
            if let Err(e) = model_file.read_exact(bytes.as_mut_slice()) {
                println!("Error reading vertex data from file: {}", e);
                return None;
            }
    
            let mut v = Vec::with_capacity(vertices_size as usize / mem::size_of::<f32>());
            for i in (0..bytes.len()).step_by(mem::size_of::<f32>()) {
                let b = [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]];
                v.push(f32::from_le_bytes(b));
            }
            v
        };
        
        let index_count = match read_u32(&mut model_file) {
            Ok(n) => { (n / mem::size_of::<u16>() as u32) as usize }
            Err(_) => { return None; }
        };
        
        let indices = match read_u16_data(&mut model_file, index_count) {
            Ok(n) => { n }
            Err(_) => { return None; }
        };
        let indices: Vec<u32> = indices.iter().map(|&n|{n as u32}).collect();
    
        let vertex_array = VertexArray {
            vertices,
            indices,
            attribute_offsets: vec![3, 3, 3, 3, 2]
        };
    
        Some(OzyMeshOBSOLETE {
            vertex_array,
            texture_name,
            uv_velocity,
            colors,
            is_transparent
        })
    }

    pub fn load_32bit_idx(path: &str) -> Option<Self> {
        let mut texture_name = String::new();
        let mut colors = vec![];

        //Open the file
        let mut model_file = match File::open(path) {
            Ok(file) => { file }
            Err(e) => {
                println!("Unable to open \"{}\": {}", path, e);
                return None;
            }
        };

        //Check how many solid colors there are
        //If 0, it means this model uses textures
        let color_count = match read_u8(&mut model_file) {
            Ok(count) => { count as usize }
            Err(e) => {
                println!("{}", e);
                return None;
            }
        };

        //Branching on whether or not the model is textured or uses solid colors
        if color_count == 0 {
            //Read the material name
            texture_name = match read_pascal_strings(&mut model_file, 1) {
                Ok(v) => { v[0].clone() }
                Err(_) => { return None; }
            };
        } else {
            //Read the color f32s into a Vec
            colors = match read_f32_data(&mut model_file, color_count * 4) {
                Ok(data_block) => { data_block }
                Err(e) => {
                    println!("Error reading color data: {}", e);
                    return None;
                }
            };
        }

        //Transparency flag
        let is_transparent = match read_u8(&mut model_file) {
            Ok(flag) => { flag != 0 }
            Err(e) => {
                println!("Error reading transparency flag: {}", e);
                return None;
            }
        };

        //The uv_velocity
        let uv_velocity = match read_f32_data(&mut model_file, 2) {
            Ok(data) => { [data[0], data[1]] }
            Err(e) => { 
                println!("Error reading uv_velocity: {}", e);
                return None;
            }
        };
    
        //The length of the vertex data section of the file, in bytes
        let vertices_size = match read_u32(&mut model_file) {
            Ok(n) => { n }
            Err(_) => { return None; }
        };
    
        let vertices = {
            let mut bytes = vec![0; vertices_size as usize];
            if let Err(e) = model_file.read_exact(bytes.as_mut_slice()) {
                println!("Error reading vertex data from file: {}", e);
                return None;
            }
    
            let mut v = Vec::with_capacity(vertices_size as usize / mem::size_of::<f32>());
            for i in (0..bytes.len()).step_by(mem::size_of::<f32>()) {
                let b = [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]];
                v.push(f32::from_le_bytes(b));
            }
            v
        };
        
        let index_count = match read_u32(&mut model_file) {
            Ok(n) => { (n / mem::size_of::<u16>() as u32) as usize }
            Err(_) => { return None; }
        };
        
        let indices = match read_u32_data(&mut model_file, index_count) {
            Ok(n) => { n }
            Err(_) => { return None; }
        };
    
        let vertex_array = VertexArray {
            vertices,
            indices,
            attribute_offsets: vec![3, 3, 3, 3, 2]
        };
    
        Some(OzyMeshOBSOLETE {
            vertex_array,
            texture_name,
            uv_velocity,
            colors,
            is_transparent
        })
    }

}

pub fn read_u32_from_le_bytes(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]])
}

pub fn read_u8(file: &mut File) -> Result<u8> {
    let mut n = [0];
    match file.read_exact(&mut n) {
        Ok(_) => { Ok(u8::from_le_bytes(n)) }
        Err(e) => { Err(e) }
    }    
}

pub fn read_u8_data(file: &mut File, count: usize) -> Result<Vec<u8>> {
    let mut bytes = vec![0; count];
    if let Err(e) = file.read_exact(bytes.as_mut_slice()) {
        return Err(e);
	}
	Ok(bytes)
}

pub fn read_u32(file: &mut File) -> Result<u32> {
	let mut buffer = [0; 4];
	match file.read_exact(&mut buffer) {
		Ok(_) => { Ok(u32::from_le_bytes(buffer)) }
		Err(e) => { Err(e) }
	}
}

pub fn read_u32_data(file: &mut File, count: usize) -> Result<Vec<u32>> {
	let mut bytes = vec![0; count * mem::size_of::<u32>()];
	if let Err(e) = file.read_exact(bytes.as_mut_slice()) {
        return Err(e);
	}

	let mut v = Vec::with_capacity(count);
	for i in (0..bytes.len()).step_by(mem::size_of::<u32>()) {
		let b = [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]];
		v.push(u32::from_le_bytes(b));
	}
	Ok(v)
}

pub fn read_u32_array(file: &mut File) -> Result<Vec<u32>> {
    let count = read_u32(file).unwrap();
    read_u32_data(file, count as usize)
}

pub fn read_u16_data(file: &mut File, count: usize) -> Result<Vec<u16>> {
	let mut bytes = vec![0; count * mem::size_of::<u16>()];
	if let Err(e) = file.read_exact(bytes.as_mut_slice()) {
        return Err(e);
	}

	let mut v = Vec::with_capacity(count);
	for i in (0..bytes.len()).step_by(mem::size_of::<u16>()) {
		let b = [bytes[i], bytes[i + 1]];
		v.push(u16::from_le_bytes(b));
	}
	Ok(v)
}

pub fn read_f32(file: &mut File) -> Result<f32> {
    let mut bytes = [0u8; 4];
	match file.read_exact(&mut bytes) {
		Ok(_) => { Ok(f32::from_le_bytes(bytes)) }
		Err(e) => { Err(e) }
	}
}

pub fn read_f32_data(file: &mut File, count: usize) -> Result<Vec<f32>> {
	let mut bytes = vec![0; count * mem::size_of::<f32>()];
	if let Err(e) = file.read_exact(bytes.as_mut_slice()) {
        return Err(e);
	}

	let mut v = Vec::with_capacity(count);
	for i in (0..bytes.len()).step_by(mem::size_of::<f32>()) {
		let b = [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]];
		v.push(f32::from_le_bytes(b));
	}
	Ok(v)    
}

pub fn read_f32_array(file: &mut File) -> Result<Vec<f32>> {
    let count = read_u32(file).unwrap();
    read_f32_data(file, count as usize)
}

pub fn read_pascal_strings(file: &mut File, count: usize) -> Result<Vec<String>> {	
	let mut int_buffer = [0; 4];
	let mut strings = Vec::with_capacity(count as usize);
	for _ in 0..count {
		//Read the name's length
		if let Err(e) = file.read_exact(&mut int_buffer) {
			return Err(e);
		}

		//Read the contents of the string into a buffer
		let mut utf8_buffer = vec![0; u32::from_le_bytes(int_buffer) as usize];
		if let Err(e) = file.read_exact(&mut utf8_buffer) {
			return Err(e);
		}

		//Turn buffer of utf-8 bytes into String
		match String::from_utf8(utf8_buffer) {
			Ok(name) => { strings.push(name); }
			Err(e) => {
				panic!("read_pascal_strings(): Error when making String from utf8: {}", e);
			}
		}
	}
	Ok(strings)
}

pub fn write_pascal_strings(file: &mut File, strs: &[&str]) -> Result<()> {
    for i in 0..strs.len() {
        let s = strs[i];
        let l = s.len() as u32;
        if let Err(e) = file.write(&u32::to_le_bytes(l)) {
            return Err(e)
        }        

        if let Err(e) = file.write(s.as_bytes()) {
            return Err(e)
        }
    }
    Ok(())
}