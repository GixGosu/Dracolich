/// Shader compilation and management
/// Handles loading GLSL shaders, compiling them, linking programs, and setting uniforms

use gl::types::*;
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::ffi::{CStr, CString};
use std::fs;
use std::ptr;

pub struct ShaderProgram {
    program_id: GLuint,
}

impl ShaderProgram {
    /// Load shader from files and create program
    pub fn from_files(vertex_path: &str, fragment_path: &str) -> Result<Self, String> {
        // Read shader source files
        let vertex_src = fs::read_to_string(vertex_path)
            .map_err(|e| format!("Failed to read vertex shader {}: {}", vertex_path, e))?;
        let fragment_src = fs::read_to_string(fragment_path)
            .map_err(|e| format!("Failed to read fragment shader {}: {}", fragment_path, e))?;

        Self::from_source(&vertex_src, &fragment_src)
    }

    /// Create shader program from source strings
    pub fn from_source(vertex_src: &str, fragment_src: &str) -> Result<Self, String> {
        unsafe {
            // Compile vertex shader
            let vertex_shader = Self::compile_shader(vertex_src, gl::VERTEX_SHADER)?;

            // Compile fragment shader
            let fragment_shader = Self::compile_shader(fragment_src, gl::FRAGMENT_SHADER)?;

            // Link program
            let program_id = gl::CreateProgram();
            gl::AttachShader(program_id, vertex_shader);
            gl::AttachShader(program_id, fragment_shader);
            gl::LinkProgram(program_id);

            // Check for linking errors
            let mut success = 0;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetProgramInfoLog(
                    program_id,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                let error = String::from_utf8_lossy(&buffer);
                return Err(format!("Shader program linking failed: {}", error));
            }

            // Clean up shaders (no longer needed after linking)
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            Ok(Self { program_id })
        }
    }

    /// Compile a single shader
    unsafe fn compile_shader(source: &str, shader_type: GLenum) -> Result<GLuint, String> {
        let shader = gl::CreateShader(shader_type);
        let c_str = CString::new(source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check for compilation errors
        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0u8; len as usize];
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut GLchar);
            let error = String::from_utf8_lossy(&buffer);
            let shader_type_name = match shader_type {
                gl::VERTEX_SHADER => "vertex",
                gl::FRAGMENT_SHADER => "fragment",
                _ => "unknown",
            };
            return Err(format!("{} shader compilation failed: {}", shader_type_name, error));
        }

        Ok(shader)
    }

    /// Use this shader program
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }

    /// Get uniform location
    fn get_uniform_location(&self, name: &str) -> GLint {
        let c_name = CString::new(name).unwrap();
        unsafe { gl::GetUniformLocation(self.program_id, c_name.as_ptr()) }
    }

    /// Set float uniform
    pub fn set_float(&self, name: &str, value: f32) {
        let location = self.get_uniform_location(name);
        if location >= 0 {
            unsafe {
                gl::Uniform1f(location, value);
            }
        }
    }

    /// Set int uniform
    pub fn set_int(&self, name: &str, value: i32) {
        let location = self.get_uniform_location(name);
        if location >= 0 {
            unsafe {
                gl::Uniform1i(location, value);
            }
        }
    }

    /// Set Vec2 uniform
    pub fn set_vec2(&self, name: &str, value: &Vec2) {
        let location = self.get_uniform_location(name);
        if location >= 0 {
            unsafe {
                gl::Uniform2f(location, value.x, value.y);
            }
        }
    }

    /// Set Vec3 uniform
    pub fn set_vec3(&self, name: &str, value: &Vec3) {
        let location = self.get_uniform_location(name);
        if location >= 0 {
            unsafe {
                gl::Uniform3f(location, value.x, value.y, value.z);
            }
        }
    }

    /// Set Vec4 uniform
    pub fn set_vec4(&self, name: &str, value: &Vec4) {
        let location = self.get_uniform_location(name);
        if location >= 0 {
            unsafe {
                gl::Uniform4f(location, value.x, value.y, value.z, value.w);
            }
        }
    }

    /// Set Mat4 uniform
    pub fn set_mat4(&self, name: &str, value: &Mat4) {
        let location = self.get_uniform_location(name);
        if location >= 0 {
            unsafe {
                gl::UniformMatrix4fv(location, 1, gl::FALSE, value.to_cols_array().as_ptr());
            }
        }
    }

    /// Get program ID
    pub fn id(&self) -> GLuint {
        self.program_id
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.program_id);
        }
    }
}
