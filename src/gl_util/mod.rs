use std::mem;
use std::ptr;
use std::str;
use std::ops::Drop;
use std::os::raw;


use gl;
use gl::types::*;


use std::ffi::CStr;

mod shader;

pub use self::shader::*;


pub fn init_gl<F>(f:F) where F:FnMut(&str) -> *const raw::c_void {
	unsafe {
		gl::load_with(f);
        let version = {
            let data = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_bytes().to_vec();
            String::from_utf8(data).unwrap()
        };

        println!("OpenGL version {}", version);

        //bind a global vertex array object
        let mut vao = mem::uninitialized();
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }
}

#[derive(Debug)]
pub struct AttributeBuffer {
	buffer : GLuint,
	size : GLint, //1,2,3,4
	count : usize,
}

impl Drop for AttributeBuffer {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteBuffers(1, &self.buffer);
		}
	}
}

unsafe fn fill_buffer<T>(buffer : GLuint, data : &[T]) {
    gl::BindBuffer(gl::COPY_WRITE_BUFFER, buffer);
    let byte_count = data.len() * mem::size_of::<T>();
    gl::BufferData(gl::COPY_WRITE_BUFFER,
        byte_count as gl::types::GLsizeiptr,
        data.as_ptr() as *const _,
        gl::STATIC_DRAW
    );
}

#[allow(non_upper_case_globals)]
static mut bound_attrs : [Option<GLuint> ; 16] = [None ; 16];

impl AttributeBuffer {
	pub fn new(dim : u32) -> AttributeBuffer {
		if dim < 1 || dim > 4 {
			panic!();
		}

    	let mut buf = unsafe { 
    		mem::uninitialized() 
    	};
    	unsafe {
    		gl::GenBuffers(1, &mut buf);
    	};
    	
    	AttributeBuffer {
    		buffer : buf,
    		size : dim as GLint,
    		count : 0,
    	}
	}

    pub fn get_dim(&self) -> usize {
        self.size as usize
    }

	pub fn fill(&mut self, data : &[f32]) {
		self.count = data.len()/(self.size as usize);
		unsafe {
			fill_buffer(self.buffer, data);
    	}
	}

	pub fn bind_to_current_vao(&self, attr : GLint) {
		unsafe {
            if Some(self.buffer) == bound_attrs[attr as usize] {
                return;
            }
            clear_attr(attr as usize);
            bound_attrs[attr as usize] = Some(self.buffer);
    		gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
    		gl::VertexAttribPointer(attr as gl::types::GLuint, self.size, gl::FLOAT, 0,
        		0,
        		ptr::null()
    		);
    		gl::EnableVertexAttribArray(attr as GLuint);
		}
	}
}

unsafe fn clear_attr(i : usize) {
    if let Some(_) = bound_attrs[i] {
        bound_attrs[i] = None;
        gl::DisableVertexAttribArray(i as GLuint);
    }
}

#[allow(non_upper_case_globals)]
static mut current_indices : GLuint = 0;

#[derive(Debug)]
pub struct IndexBuffer {
	buffer : GLuint,
	count : usize,
	topology : GLenum,
}

impl Drop for IndexBuffer {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteBuffers(1, &self.buffer);
		}
	}
}

impl IndexBuffer {
	pub fn new() -> IndexBuffer {
    	let mut buf = unsafe { 
    		mem::uninitialized() 
    	};
    	unsafe {
    		gl::GenBuffers(1, &mut buf);
    	};
    	
    	IndexBuffer {
    		buffer : buf,
    		count : 0,
    		topology : gl::TRIANGLES,
    	}
	}

	pub fn fill(&mut self, topology : GLenum, data : &[u32]) {
		self.count = data.len();
		self.topology = topology;
		unsafe {
    		fill_buffer(self.buffer, data);
    	}
	}

	pub fn draw(&self) {
		unsafe {

			if current_indices != self.buffer {
				current_indices = self.buffer;
				gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.buffer);
			}

			gl::DrawElements(
            	self.topology,
            	self.count as GLint,
            	gl::UNSIGNED_INT,
           		ptr::null()
        	);
		}
	}
}

pub struct DrawCall<'a> {
    program : Option<&'a ShaderProgram>,
    indices : Option<&'a IndexBuffer>,
    attrib_count : usize,
    attribs : [Option<(&'a str, &'a AttributeBuffer)> ; 16],
}

impl<'a> DrawCall<'a> {
    pub fn new() -> Self {
        DrawCall {
            program : None,
            indices : None,
            attribs : [None;16],
            attrib_count : 0,
        }
    }

    pub fn set_indices(&mut self, indices : &'a IndexBuffer) {
        self.indices = Some(indices);
    }

    pub fn set_program(&mut self, program : &'a ShaderProgram) {
        self.program = Some(program);
    }

    pub fn add_attrs(&mut self, name : &'a str, attrs : &'a AttributeBuffer) {
        assert!(self.attrib_count < 16);

        self.attribs[self.attrib_count] = Some((name,attrs));
        self.attrib_count += 1;
    }

    pub fn draw(&self) {
        unsafe {
            let mut should_delete = [true ; 16];
            if !self.program.is_some() {
                return;
            }

            if !self.indices.is_some() {
                return;
            }
            let prog = self.program.unwrap();
            prog.bind();
            for i in 0..self.attrib_count {
                let (name,attrs) = self.attribs[i].unwrap();
                let index = prog.get_attr(name);
                attrs.bind_to_current_vao(index);
                should_delete[index as usize] = false;
            }

            for i in 0..16 {
                if !should_delete[i] {
                    continue
                }
                bound_attrs[i] = None;
                gl::DisableVertexAttribArray(i as GLuint);
            }

            self.indices.unwrap().draw();
        }
    }
}