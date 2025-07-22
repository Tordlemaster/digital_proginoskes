use std::{fs::File, io::{BufRead, BufReader, Read}, ops::{Add, Mul, Sub}};

use gl;
use glam::{vec3, Mat3, Vec3};

const planeVertices: [f32; 8] = [
	0.0, 1.0,
	0.0, 0.0,
	1.0, 1.0,
	1.0, 0.0,
];

const planeTexCoords: [f32; 8] = [
	0.0, 1.0,
	0.0, 0.0,
	1.0, 1.0,
	1.0, 0.0
];

const planeIndices: [u32; 6] = [
	0, 1, 2,
	1, 3, 2
];

pub static mut QUAD_VBO: u32 = 0;
pub static mut QUAD_VAO: u32 = 0;
pub static mut QUAD_EBO: u32 = 0;

pub static mut DEBUG_QT_PROGRAM: u32 = 0;
pub static mut DEBUG_QT_UNI_TRANS: gl::types::GLint = 0;
pub static mut DEBUG_QT_UNI_STARS: gl::types::GLint = 0;
pub static mut DEBUG_QT_UNI_DEPTH: gl::types::GLint = 0;

pub fn init_utils() {
    unsafe {//TODO REMOVE INTERLEAVING TO IMPROVE CODE READABILITY
        println!("a");
        gl::GenVertexArrays(1, &raw mut QUAD_VAO);
        println!("b");
        gl::BindVertexArray(QUAD_VAO);
        println!("c");
        gl::GenBuffers(1, &raw mut QUAD_VBO);
        println!("d");
        gl::BindBuffer(gl::ARRAY_BUFFER, QUAD_VBO);
        println!("e");
        gl::BufferData(gl::ARRAY_BUFFER, (size_of_val(&planeVertices) + size_of_val(&planeTexCoords)).try_into().unwrap(), std::ptr::null(), gl::STATIC_DRAW);
        println!("f");
        gl::BufferSubData(gl::ARRAY_BUFFER, 0, size_of_val(&planeVertices).try_into().unwrap(), planeVertices.as_ptr() as *const core::ffi::c_void);
        println!("g");
        gl::BufferSubData(gl::ARRAY_BUFFER, size_of_val(&planeVertices).try_into().unwrap(), size_of_val(&planeTexCoords).try_into().unwrap(), planeTexCoords.as_ptr() as *const core::ffi::c_void);
        println!("h");
        gl::GenBuffers(1, &raw mut QUAD_EBO);
        println!("i");
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, QUAD_EBO);
        println!("j");
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size_of_val(&planeIndices).try_into().unwrap(), planeIndices.as_ptr() as *const core::ffi::c_void, gl::STATIC_DRAW);
        println!("k");
        gl::EnableVertexAttribArray(0);
        println!("l");
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, (size_of::<f32>() * 2).try_into().unwrap(), 0 as *const core::ffi::c_void);
        println!("m");
        gl::EnableVertexAttribArray(1);
        println!("n");
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (size_of::<f32>() * 2).try_into().unwrap(), size_of_val(&planeVertices) as *const core::ffi::c_void);
        println!("o");
    }
}

pub fn load_shader(shader_type: gl::types::GLenum, source_filepath: &str) -> u32 {
    let mut id = 0;
    let mut shader_code = String::new();

    let mut file = File::open(source_filepath).expect("Failed to open shader source file");
    file.read_to_string(&mut shader_code).expect("Shader code invalid");
    shader_code.push_str("\0");

    let code_ptr = shader_code.as_ptr() as *const i8;

    unsafe {
        println!("pa");
        id = gl::CreateShader(shader_type);
        println!("pb");
        gl::ShaderSource(id, 1, &raw const code_ptr, std::ptr::null());
        println!("pc");
        gl::CompileShader(id);
        println!("pd");

        let mut success: gl::types::GLint = 0;
        let mut info_log: [gl::types::GLchar; 512] = [0; 512];
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &raw mut success);
        if success == 0 {
            gl::GetShaderInfoLog(id, 512, std::ptr::null_mut(), (&raw mut info_log) as *mut gl::types::GLchar);
            panic!("SHADER COMPILATION FAILED\n{}", String::from_iter(info_log.iter().map(|&c| c as u8 as char)));
        }
    }
    println!("p");
    id
}

pub fn load_shader_program(shaders: Vec<gl::types::GLuint>) -> gl::types::GLuint {
    let mut program_id: gl::types::GLuint = 0;
    unsafe {
        program_id = gl::CreateProgram();
        for s in &shaders {
            gl::AttachShader(program_id, *s);
        }
        gl::LinkProgram(program_id);

        let mut success: gl::types::GLint = 0;
        let mut info_log: [gl::types::GLchar; 512] = [0; 512];
        gl::GetProgramiv(program_id, gl::LINK_STATUS, &raw mut success);
        if success == 0 {
            gl::GetProgramInfoLog(program_id, 512, std::ptr::null_mut(), (&raw mut info_log) as *mut gl::types::GLchar);
            println!("PROGRAM LINKING FAILED\n{}", String::from_iter(info_log.iter().map(|&c| c as u8 as char)));
        }

        for s in shaders {
            gl::DeleteShader(s);
        }
    }
    println!("q");
    program_id
}

pub fn setup_debug_qt_program() {
    unsafe {
        DEBUG_QT_PROGRAM = load_shader_program(Vec::from([load_shader(gl::VERTEX_SHADER, "./src/render/shaders/debug_qt.vert"), load_shader(gl::FRAGMENT_SHADER, "./src/render/shaders/debug_qt.frag")]));
        DEBUG_QT_UNI_TRANS = gl::GetUniformLocation(DEBUG_QT_PROGRAM, "transformation".as_ptr() as *const i8);
        DEBUG_QT_UNI_STARS = gl::GetUniformLocation(DEBUG_QT_PROGRAM, "star_count".as_ptr() as *const i8);
        DEBUG_QT_UNI_DEPTH = gl::GetUniformLocation(DEBUG_QT_PROGRAM, "layer".as_ptr() as *const i8);
    }
}

pub struct BlackbodyTable {
    table: Vec<XYColor>
}

impl BlackbodyTable {
    pub fn temp_to_xy(&self, i: f32) -> XYColor {
        let i = i - 1000.0;
        let div = i / 100.0;
        let floor = div.floor();
        let below = self.table[floor as usize];
        let above = self.table[floor as usize + 1];

        below + (above - below) * (div - floor)
    }
}

#[derive(Clone, Copy)]
pub struct XYColor {
    x: f32,
    y: f32
}

impl XYColor {
    pub fn to_rgb(&self) -> Vec3 {
        let z = 1.0 - self.x - self.y;
        let Y = 1.0;
        let X = (Y / self.y) * self.x;
        let Z = (Y / self.y) * z;
        const XYZ_TO_RGB: Mat3 = Mat3::from_cols(vec3(3.2404542, -0.9692660, 0.0556434), vec3(-1.5371385, 1.8760108, -0.2040259), vec3(-0.4985314, 0.0415560, 1.0572252));

        XYZ_TO_RGB * vec3(X, Y, Z)
    }
}

impl Add for XYColor {
    type Output = XYColor;
    fn add(self, rhs: Self) -> Self::Output {
        XYColor{x: self.x + rhs.x, y: self.y + rhs.y}
    }
}

impl Sub for XYColor {
    type Output = XYColor;
    fn sub(self, rhs: Self) -> Self::Output {
        XYColor{x: self.x - rhs.x, y: self.y - rhs.y}
    }
}

impl Mul<f32> for XYColor {
    type Output = XYColor;
    fn mul(self, rhs: f32) -> Self::Output {
        XYColor{x: self.x * rhs, y: self.y * rhs}
    }
}


pub fn load_blackbody_table() -> BlackbodyTable {
    let bbr_color = File::open("./data/tables/bbr_color.txt").expect("Could not open bbr_color.txt");
    let mut buf = BufReader::new(bbr_color);
    let mut s = String::new();
    for _ in 0..=19 {
        //go past the file's header
        buf.read_line(&mut s);
    }
    let mut v = Vec::with_capacity(391);
    while buf.read_line(&mut s).unwrap() > 0 {
        s.clear();
        buf.read_line(&mut s);
        let entry: Vec<&str> = s.split_whitespace().collect();
        v.push(XYColor { x: entry[3].parse::<f32>().unwrap(), y: entry[4].parse::<f32>().unwrap() });
        if entry[0] == "40000" {
            break;
        }
    }
    BlackbodyTable { table: v }
}

pub fn ra_dec_to_xyz(ra: f32, dec: f32) -> (f32, f32, f32) {
    let a = vec3(
        dec.cos() * ra.sin(),
        dec.sin(),
        dec.cos() * ra.cos(),
    );
    (a.x, a.y, a.z)
}

///Convert angle with arc minutes and seconds to a decimal angle
pub fn deg_ams(deg: f32, min: f32, sec: f32) -> f32 {
    deg + (min * (1.0/60.0)) + (sec * (1.0/3600.0))
}