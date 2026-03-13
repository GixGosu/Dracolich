/// Safe Rust wrappers around raw OpenGL objects
/// Provides RAII-style resource management for VAOs, VBOs, EBOs, and textures

use gl::types::*;
use std::ffi::c_void;

/// Vertex Array Object (VAO) wrapper
pub struct VAO {
    id: GLuint,
}

impl VAO {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }
        Self { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for VAO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

/// Vertex Buffer Object (VBO) wrapper
pub struct VBO {
    id: GLuint,
    buffer_type: GLenum,
}

impl VBO {
    pub fn new(buffer_type: GLenum) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        Self { id, buffer_type }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(self.buffer_type, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(self.buffer_type, 0);
        }
    }

    /// Upload data to the buffer
    pub fn upload_data<T>(&self, data: &[T], usage: GLenum) {
        self.bind();
        unsafe {
            gl::BufferData(
                self.buffer_type,
                (data.len() * std::mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
                usage,
            );
        }
    }

    /// Update a portion of the buffer
    pub fn update_data<T>(&self, offset: usize, data: &[T]) {
        self.bind();
        unsafe {
            gl::BufferSubData(
                self.buffer_type,
                offset as GLintptr,
                (data.len() * std::mem::size_of::<T>()) as GLsizeiptr,
                data.as_ptr() as *const c_void,
            );
        }
    }

    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for VBO {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

/// Element Buffer Object (EBO) wrapper - specialized VBO for indices
pub type EBO = VBO;

/// Texture wrapper
pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }
        Self { id }
    }

    pub fn bind(&self, target: GLenum) {
        unsafe {
            gl::BindTexture(target, self.id);
        }
    }

    pub fn unbind(&self, target: GLenum) {
        unsafe {
            gl::BindTexture(target, 0);
        }
    }

    /// Upload 2D texture data
    pub fn upload_2d(
        &self,
        width: i32,
        height: i32,
        internal_format: GLint,
        format: GLenum,
        data_type: GLenum,
        data: *const c_void,
    ) {
        self.bind(gl::TEXTURE_2D);
        unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format,
                width,
                height,
                0,
                format,
                data_type,
                data,
            );
        }
    }

    /// Set texture parameters
    pub fn set_parameter(&self, target: GLenum, pname: GLenum, param: GLint) {
        self.bind(target);
        unsafe {
            gl::TexParameteri(target, pname, param);
        }
    }

    /// Generate mipmaps
    pub fn generate_mipmap(&self, target: GLenum) {
        self.bind(target);
        unsafe {
            gl::GenerateMipmap(target);
        }
    }

    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

/// Helper function to enable and configure a vertex attribute
pub fn enable_vertex_attrib(
    index: GLuint,
    size: GLint,
    data_type: GLenum,
    normalized: bool,
    stride: GLsizei,
    offset: usize,
) {
    unsafe {
        gl::EnableVertexAttribArray(index);
        gl::VertexAttribPointer(
            index,
            size,
            data_type,
            if normalized { gl::TRUE } else { gl::FALSE },
            stride,
            offset as *const c_void,
        );
    }
}

/// Helper to check for OpenGL errors (debug only)
#[cfg(debug_assertions)]
pub fn check_gl_error(context: &str) {
    unsafe {
        let err = gl::GetError();
        if err != gl::NO_ERROR {
            eprintln!("OpenGL error in {}: 0x{:X}", context, err);
        }
    }
}

#[cfg(not(debug_assertions))]
pub fn check_gl_error(_context: &str) {}
