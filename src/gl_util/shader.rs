use std::ptr;
use std::str;
use std::ops::Drop;


use gl;
use gl::types::*;


use std::ffi::CStr;
use std::ffi::CString;

type ShaderSrc = (&'static [u8], &'static [u8]);


#[allow(non_upper_case_globals)]
static mut current_prog : GLuint = 0;

#[derive(Debug)]
pub struct ShaderProgram {
    prog : GLuint,
    pub inputs : Vec<ShaderAttributeInfo>,
}

impl Drop for ShaderProgram {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteProgram(self.prog);
		}
	}
}

#[derive(Debug)]
pub struct ShaderAttributeInfo {
    pub name : String,
    pub location : GLuint,
    pub dim : usize,
}

unsafe fn get_shader(shader : GLuint, property : GLuint) -> GLint {
    let mut result : gl::types::GLint = 0;
    gl::GetShaderiv(shader, property, &mut result);
    result
}

unsafe fn get_program(prog : GLuint, property : GLuint) -> GLint {
    let mut result : gl::types::GLint = 0;
    gl::GetProgramiv(prog, property, &mut result);
    result
}

unsafe fn check_shader_errors(shader : GLuint) {
    let status = get_shader(shader, gl::COMPILE_STATUS);

    if status == gl::FALSE as GLint {
        println!("err");
        let len = get_shader(shader, gl::INFO_LOG_LENGTH) as usize;
        let mut buf = Vec::with_capacity(len);
        buf.set_len(len -1);
        gl::GetShaderInfoLog(shader, len as GLint, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        println!("{:?}", str::from_utf8(&buf).ok().unwrap());
    }
}

unsafe fn get_program_iv(prog : GLuint, pname : GLenum) -> GLint {
    let mut result : gl::types::GLint = 0;
    gl::GetProgramiv(prog, pname, &mut result);
    result
}

unsafe fn get_active_attribute(prog : GLuint, index : GLuint, name_buf : &mut [u8]) -> ShaderAttributeInfo {
    let max_len = name_buf.len() as GLsizei;
    let mut size : GLint = 0;
    let mut data_type : GLenum = 0;
    let mut name_len : GLsizei = 0;
    gl::GetActiveAttrib(
        prog, 
        index, 
        max_len, 
        &mut name_len, 
        &mut size, 
        &mut data_type, 
        name_buf.as_mut_ptr() as *mut GLchar
    );

    let dim = match data_type {
        gl::FLOAT => 1,
        gl::FLOAT_VEC2 => 2,
        gl::FLOAT_VEC3 => 3,
        gl::FLOAT_VEC4 => 4,
        _ => panic!(),
    };

    assert_eq!(1, size);

    ShaderAttributeInfo {
        name : glstr_to_string(name_buf.as_ptr() as *const GLubyte),
        location : index,
        dim : dim,
    }
}


unsafe fn glstr_to_string(ptr : *const GLubyte) -> String {
    if ptr == ptr::null() {
        panic!();
    }
    let data = CStr::from_ptr(ptr as *const _).to_bytes().to_vec();
    String::from_utf8(data).unwrap()
}

impl ShaderProgram {
	pub fn build_program(src : ShaderSrc) -> ShaderProgram {
		unsafe {
    		let vs = gl::CreateShader(gl::VERTEX_SHADER);
    		assert!(0 != vs);
    		gl::ShaderSource(vs, 1, [src.0.as_ptr() as *const _].as_ptr(), ptr::null());
    		gl::CompileShader(vs);
    		check_shader_errors(vs);
		
    		let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
    		gl::ShaderSource(fs, 1, [src.1.as_ptr() as *const _].as_ptr(), ptr::null());
    		gl::CompileShader(fs);
    		check_shader_errors(fs);
		
    		let prog = gl::CreateProgram();
    		assert!(0 != prog);
    		gl::AttachShader(prog, vs);
    		if gl::NO_ERROR != gl::GetError() {
    		    println!("shader attach error");
    		}
    		gl::DeleteShader(vs);
    		gl::AttachShader(prog, fs);
    		if gl::NO_ERROR != gl::GetError() {
    		    println!("shader attach error");
    		}
    		gl::DeleteShader(fs);
		
    		gl::LinkProgram(prog);
    		if gl::TRUE as GLint != get_program(prog, gl::LINK_STATUS) {
    		    let len = get_program(prog, gl::INFO_LOG_LENGTH) as usize;
    		    let mut buf = Vec::with_capacity(len);
    		    buf.set_len(len -1);
    		    gl::GetProgramInfoLog(prog, len as GLint, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
    		    println!("{:?}", str::from_utf8(&buf).ok().unwrap());
    		}
		
    		//query attributes
    		let input_count = get_program_iv(prog, gl::ACTIVE_ATTRIBUTES) as u32;
    		let input_name_len = get_program_iv(prog, gl::ACTIVE_ATTRIBUTE_MAX_LENGTH);
		
    		let mut input_name_buf : Vec<u8> = Vec::with_capacity(input_name_len as usize);
    		for _ in 0..input_name_len {
    		    input_name_buf.push(0);
    		}
		
    		let mut inputs = Vec::with_capacity(input_count as usize);
    		for input_index in 0..input_count {
    		    let input = get_active_attribute(prog, input_index, &mut input_name_buf[..]);
    		    println!("{:?}", input);
    		    inputs.push(input);
    		}
		
    		ShaderProgram {
    		    prog : prog,
    		    inputs : inputs,
    		}
    	}
    }

    pub fn get_attr(&self, name : &str) -> GLint {
    	unsafe {
    		let s = CString::new(name).unwrap();
    		gl::GetAttribLocation(self.prog, s.as_ptr() as *const _)
		}
	}

	pub fn bind(&self) {
		unsafe {
			if current_prog != self.prog {
				current_prog = self.prog;
				gl::UseProgram(self.prog);
			}
		}
	}
}
