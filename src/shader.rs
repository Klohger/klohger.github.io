use std::fmt::Debug;

use crate::WebGLProgram;

use super::println;

use super::GL;

use super::WebGLShader;

#[derive(Debug)]
pub enum ShaderProgramCreationError {
    Linking(String),
    Validation(String),
}

impl std::fmt::Display for ShaderProgramCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
impl std::error::Error for ShaderProgramCreationError {}
#[repr(transparent)]
pub struct ShaderProgram {
    id: WebGLProgram,
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        GL.with(|gl| gl.delete_program(&self.id))
    }
}
impl ShaderProgram {
    pub unsafe fn new(
        vertex: &Shader,
        fragment: &Shader,
    ) -> Result<Self, ShaderProgramCreationError> {
        GL.with(|gl| {
            let program = gl.create_program();
            gl.attach_shader(&program, &vertex.id);
            gl.attach_shader(&program, &fragment.id);
            gl.link_program(&program);

            let info_log = gl.get_program_info_log(&program);
            if gl.get_program_parameter_bool(&program, gl.link_status()) {
                gl.validate_program(&program);
                if gl.get_program_parameter_bool(&program, gl.validate_status()) {
                    if info_log.len() > 0 {
                        println(&info_log)
                    }
                    Ok(Self { id: program })
                } else {
                    Err(ShaderProgramCreationError::Validation(info_log))
                }
            } else {
                Err(ShaderProgramCreationError::Linking(info_log))
            }
        })
    }
}
#[repr(transparent)]
pub struct Shader {
    id: WebGLShader,
}

impl Drop for Shader {
    fn drop(&mut self) {
        GL.with(|gl| gl.delete_shader(&self.id))
    }
}

impl Shader {
    pub fn new(r#type: u32, source: &str) -> Result<Self, String> {
        GL.with(|gl| {
            let shader = gl.create_shader(r#type);
            gl.set_shader_source(&shader, source);
            gl.compile_shader(&shader);
            let info_log = gl.get_shader_info_log(&shader);
            if gl.get_shader_parameter_bool(&shader, gl.compile_status()) {
                if info_log.len() > 0 {
                    println(&info_log)
                }
                Ok(Self { id: shader })
            } else {
                Err(info_log)
            }
        })
    }
    pub fn id(&self) -> &WebGLShader {
        &self.id
    }
}
