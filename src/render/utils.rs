use std::{fs::File, io::Read};

use gl;

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