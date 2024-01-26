use core::{arch::wasm32, slice};

use shader::{Shader, ShaderProgram};
use slab::Slab;
use std::{
    arch::wasm32::{i32x4_extract_lane, v128},
    cell::{Ref, RefCell},
    marker::PhantomData,
    mem, panic,
};
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsValue};
mod shader;

#[wasm_bindgen]
extern "C" {

    type Window;

    #[wasm_bindgen(structural, method, js_class = "Window", js_name = addEventListener)]
    fn add_event_listener(this: &Window, event: &str, function: &Closure<dyn Fn()>);

    # [wasm_bindgen(structural , method , getter, js_class = "Window" , js_name = innerWidth)]
    fn width(this: &Window) -> u32;

    # [wasm_bindgen(structural , method , getter, js_class = "Window" , js_name = innerHeight)]
    fn height(this: &Window) -> u32;

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn println(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn println_js(js: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn eprintln(s: &str);

    #[wasm_bindgen(js_name = window)]
    static WINDOW: Window;

    type Document;
    #[wasm_bindgen(js_name = document)]
    static DOCUMENT: Document;

    type Element;

    #[wasm_bindgen(js_name = querySelector, method)]
    fn query_selector(this: &Document, query: &str) -> Option<Element>;

    type HTMLCanvasElement;

    #[wasm_bindgen(js_name = getContext, method)]
    fn get_context(this: &HTMLCanvasElement, query: &str) -> Option<WebGLRenderingContext>;
    #[wasm_bindgen(method, setter)]
    fn set_width(this: &HTMLCanvasElement, value: u32);
    #[wasm_bindgen(method, setter)]
    fn set_height(this: &HTMLCanvasElement, value: u32);

    type WebGLRenderingContext;

    #[wasm_bindgen(method, js_name = clearColor)]
    fn clear_color(this: &WebGLRenderingContext, red: f32, green: f32, blue: f32, alpha: f32);

    #[wasm_bindgen(method)]
    fn clear(this: &WebGLRenderingContext, bitfield: u32);

    #[wasm_bindgen(method, getter, js_name=COLOR_BUFFER_BIT)]
    fn color_buffer_bit(this: &WebGLRenderingContext) -> u32;

    type WebGLShader;
    #[wasm_bindgen(method, js_name = createShader)]
    fn create_shader(this: &WebGLRenderingContext, r#type: u32) -> WebGLShader;

    #[wasm_bindgen(getter, method, js_name = VERTEX_SHADER)]
    fn vertex_shader(this: &WebGLRenderingContext) -> u32;

    #[wasm_bindgen(getter, method, js_name = FRAGMENT_SHADER)]
    fn fragment_shader(this: &WebGLRenderingContext) -> u32;

    #[wasm_bindgen(method, js_name = shaderSource)]
    fn set_shader_source(this: &WebGLRenderingContext, shader: &WebGLShader, source: &str);

    #[wasm_bindgen(method, js_name = compileShader)]
    fn compile_shader(this: &WebGLRenderingContext, shader: &WebGLShader);

    #[wasm_bindgen(method, js_name = deleteShader)]
    fn delete_shader(this: &WebGLRenderingContext, shader: &WebGLShader);

    #[wasm_bindgen(method, js_name = getShaderInfoLog)]
    fn get_shader_info_log(this: &WebGLRenderingContext, shader: &WebGLShader) -> String;

    #[wasm_bindgen(method, js_name = getShaderParameter)]
    fn get_shader_parameter_bool(
        this: &WebGLRenderingContext,
        shader: &WebGLShader,
        parameter: usize,
    ) -> bool;

    #[wasm_bindgen(getter, method, js_name = COMPILE_STATUS)]
    fn compile_status(this: &WebGLRenderingContext) -> usize;

    #[wasm_bindgen(getter, method, js_name = SHADER_TYPE)]
    fn shader_type(this: &WebGLRenderingContext) -> usize;

    type WebGLProgram;
    #[wasm_bindgen(method, js_name = createProgram)]
    fn create_program(this: &WebGLRenderingContext) -> WebGLProgram;
    #[wasm_bindgen(method, js_name = attachShader)]
    fn attach_shader(this: &WebGLRenderingContext, program: &WebGLProgram, shader: &WebGLShader);

    #[wasm_bindgen(method, js_name = linkProgram)]
    fn link_program(this: &WebGLRenderingContext, program: &WebGLProgram);

    #[wasm_bindgen(method, js_name = validateProgram)]
    fn validate_program(this: &WebGLRenderingContext, program: &WebGLProgram);

    #[wasm_bindgen(method, js_name = getProgramInfoLog)]
    fn get_program_info_log(this: &WebGLRenderingContext, program: &WebGLProgram) -> String;

    #[wasm_bindgen(method, js_name = getProgramParameter)]
    fn get_program_parameter_bool(
        this: &WebGLRenderingContext,
        program: &WebGLProgram,
        parameter: usize,
    ) -> bool;

    #[wasm_bindgen(getter, method, js_name = LINK_STATUS)]
    fn link_status(this: &WebGLRenderingContext) -> usize;
    #[wasm_bindgen(getter, method, js_name = VALIDATE_STATUS)]
    fn validate_status(this: &WebGLRenderingContext) -> usize;

    #[wasm_bindgen(method, js_name = deleteProgram)]
    fn delete_program(this: &WebGLRenderingContext, program: &WebGLProgram);

    type WebGLUniformLocation;
}

const VERTEX_SHADER_SOURCE: &str = include_str!("test.vert");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("test.frag");
thread_local! {
    static REFRESH_CANVAS_SIZE : Closure<dyn Fn()> = Closure::new(|| {
        refresh_canvas_size()
    });
    static CANVAS : HTMLCanvasElement =
        HTMLCanvasElement::from(DOCUMENT.query_selector("canvas").expect("failed to access canvas element").obj);
    static GL : WebGLRenderingContext = CANVAS.with(|canvas| {
        canvas
            .get_context("webgl")
            .expect("failed to get webgl rendering context")
    });
}
static mut SHADER_PROGRAMS: ShaderProgramManager = ShaderProgramManager::new();

fn refresh_canvas_size() {
    CANVAS.with(|canvas| {
        canvas.set_width(WINDOW.width());
        canvas.set_height(WINDOW.height());
    })
}
struct ShaderListElement {
    pub shader: Shader,
    pub usage: usize,
}

struct ShaderProgramListElement {
    pub vertex_shader_id: usize,
    pub fragment_shader_id: usize,
    pub shader_program: ShaderProgram,
}

struct ShaderProgramIdenitier {
    shader_program_id: usize,
    vertex_shader_id: usize,
    fragment_shader_id: usize,
}

struct ShaderProgramManager {
    shaders: Slab<ShaderListElement>,
    shader_programs: Slab<ShaderProgramListElement>,
}

impl ShaderProgramManager {
    pub const fn new() -> ShaderProgramManager {
        Self {
            shaders: Slab::new(),
            shader_programs: Slab::new(),
        }
    }

    pub fn program(&self, id: usize) -> &ShaderProgram {
        &self.shader_programs[id].shader_program
    }

    pub fn new_program_id_id(
        &mut self,
        vertex_shader_id: usize,
        fragment_shader_id: usize,
    ) -> ShaderProgramIdenitier {
        let shader_program = unsafe {
            ShaderProgram::new(
                &self.shaders[vertex_shader_id].shader,
                &self.shaders[fragment_shader_id].shader,
            )
            .unwrap()
        };
        let shader_program_id = self.shader_programs.insert(ShaderProgramListElement {
            vertex_shader_id,
            fragment_shader_id,
            shader_program,
        });
        self.shaders[vertex_shader_id].usage += 1;
        self.shaders[fragment_shader_id].usage += 1;
        ShaderProgramIdenitier {
            shader_program_id,
            vertex_shader_id,
            fragment_shader_id,
        }
    }
    pub fn new_program_source_id<V: AsRef<str>>(
        &mut self,
        vertex_shader_source: V,
        fragment_shader_id: usize,
    ) -> ShaderProgramIdenitier {
        let vertex_shader_id = self.new_vertex_shader(vertex_shader_source.as_ref());
        let shader_program = unsafe {
            ShaderProgram::new(
                &self.shaders[vertex_shader_id].shader,
                &self.shaders[fragment_shader_id].shader,
            )
            .unwrap()
        };
        let shader_program_id = self.shader_programs.insert(ShaderProgramListElement {
            vertex_shader_id,
            fragment_shader_id,
            shader_program,
        });
        self.shaders[vertex_shader_id].usage += 1;
        self.shaders[fragment_shader_id].usage += 1;
        ShaderProgramIdenitier {
            shader_program_id,
            vertex_shader_id,
            fragment_shader_id,
        }
    }
    pub fn new_program_id_source<F: AsRef<str>>(
        &mut self,
        vertex_shader_id: usize,
        fragment_shader_source: F,
    ) -> ShaderProgramIdenitier {
        let fragment_shader_id = self.new_fragment_shader(fragment_shader_source.as_ref());
        let shader_program = unsafe {
            ShaderProgram::new(
                &self.shaders[vertex_shader_id].shader,
                &self.shaders[fragment_shader_id].shader,
            )
            .unwrap()
        };
        let shader_program_id = self.shader_programs.insert(ShaderProgramListElement {
            vertex_shader_id,
            fragment_shader_id,
            shader_program,
        });
        self.shaders[vertex_shader_id].usage += 1;
        self.shaders[fragment_shader_id].usage += 1;
        ShaderProgramIdenitier {
            shader_program_id,
            vertex_shader_id,
            fragment_shader_id,
        }
    }
    pub fn new_program_source_source(
        &mut self,
        vertex_shader_source: impl AsRef<str>,
        fragment_shader_source: impl AsRef<str>,
    ) -> ShaderProgramIdenitier {
        let vertex_shader_id = self.new_vertex_shader(vertex_shader_source.as_ref());
        let fragment_shader_id = self.new_fragment_shader(fragment_shader_source.as_ref());
        let shader_program = unsafe {
            ShaderProgram::new(
                &self.shaders[vertex_shader_id].shader,
                &self.shaders[fragment_shader_id].shader,
            )
            .unwrap()
        };
        let shader_program_id = self.shader_programs.insert(ShaderProgramListElement {
            vertex_shader_id,
            fragment_shader_id,
            shader_program,
        });
        self.shaders[vertex_shader_id].usage += 1;
        self.shaders[fragment_shader_id].usage += 1;
        ShaderProgramIdenitier {
            shader_program_id,
            vertex_shader_id,
            fragment_shader_id,
        }
    }
    pub fn remove_program(&mut self, shader_program_id: usize) {
        let entry = &self.shader_programs[shader_program_id];
        if self.shaders[entry.vertex_shader_id].usage == 1 {
            self.shaders.remove(entry.vertex_shader_id);
        } else {
            self.shaders[entry.vertex_shader_id].usage -= 1;
        }
        if self.shaders[entry.fragment_shader_id].usage == 1 {
            self.shaders.remove(entry.fragment_shader_id);
        } else {
            self.shaders[entry.fragment_shader_id].usage -= 1;
        }
        self.shader_programs.remove(shader_program_id);
    }

    fn new_vertex_shader(&mut self, source: &str) -> usize {
        GL.with(|gl| {
            self.shaders.insert(ShaderListElement {
                shader: Shader::new(gl.vertex_shader(), source).unwrap(),
                usage: 0,
            })
        })
    }
    fn new_fragment_shader(&mut self, source: &str) -> usize {
        GL.with(|gl| {
            self.shaders.insert(ShaderListElement {
                shader: Shader::new(gl.fragment_shader(), source).unwrap(),
                usage: 0,
            })
        })
    }
}
#[repr(transparent)]
struct Vec3<T>(v128, PhantomData<T>);
#[repr(transparent)]
struct Vec2<T>(v128, PhantomData<T>);
#[repr(transparent)]
struct Vec4<T>(v128, PhantomData<T>);

enum UniformValue<'a> {
    I32(&'a i32),
    I32Array(&'a [i32]),
    I32_2(&'a [i32; 2]),
    I32_2Array(&'a [[i32; 2]]),
    I32_3(&'a [i32; 3]),
    I32_3Array(&'a [[i32; 3]]),
    I32_4(&'a [i32; 4]),
    I32_4Array(&'a [[i32; 4]]),

    F32(&'a f32),
    F32Array(&'a [f32]),
    F32_2(&'a [f32; 2]),
    F32_2Array(&'a [[f32; 2]]),
    F32_3(&'a [f32; 3]),
    F32_3Array(&'a [[f32; 3]]),
    F32_4(&'a [f32; 4]),
    F32_4Array(&'a [[f32; 4]]),
    Mat2(&'a [f32; 2 * 2]),
    Mat3(&'a [f32; 3 * 3]),
    Mat4(&'a [f32; 4 * 4]),
}

impl<'a> From<&'a i32> for UniformValue<'a> {
    fn from(value: &'a i32) -> Self {
        UniformValue::I32(value)
    }
}
impl<'a> From<&'a [i32]> for UniformValue<'a> {
    fn from(value: &'a [i32]) -> Self {
        UniformValue::I32Array(value)
    }
}
impl<'a> From<&'a [i32; 2]> for UniformValue<'a> {
    fn from(value: &'a [i32; 2]) -> Self {
        UniformValue::I32_2(value)
    }
}
impl<'a> From<&'a [[i32; 2]]> for UniformValue<'a> {
    fn from(value: &'a [[i32; 2]]) -> Self {
        UniformValue::I32_2Array(value)
    }
}
impl<'a> From<&'a [i32; 3]> for UniformValue<'a> {
    fn from(value: &'a [i32; 3]) -> Self {
        UniformValue::I32_3(value)
    }
}
impl<'a> From<&'a [[i32; 3]]> for UniformValue<'a> {
    fn from(value: &'a [[i32; 3]]) -> Self {
        UniformValue::I32_3Array(value)
    }
}
impl<'a> From<&'a [i32; 4]> for UniformValue<'a> {
    fn from(value: &'a [i32; 4]) -> Self {
        UniformValue::I32_4(value)
    }
}
impl<'a> From<&'a [[i32; 4]]> for UniformValue<'a> {
    fn from(value: &'a [[i32; 4]]) -> Self {
        UniformValue::I32_4Array(value)
    }
}

impl<'a> From<&'a f32> for UniformValue<'a> {
    fn from(value: &'a f32) -> Self {
        UniformValue::F32(value)
    }
}
impl<'a> From<&'a [f32]> for UniformValue<'a> {
    fn from(value: &'a [f32]) -> Self {
        UniformValue::F32Array(value)
    }
}
impl<'a> From<&'a [f32; 2]> for UniformValue<'a> {
    fn from(value: &'a [f32; 2]) -> Self {
        UniformValue::F32_2(value)
    }
}
impl<'a> From<&'a [[f32; 2]]> for UniformValue<'a> {
    fn from(value: &'a [[f32; 2]]) -> Self {
        UniformValue::F32_2Array(value)
    }
}
impl<'a> From<&'a [f32; 3]> for UniformValue<'a> {
    fn from(value: &'a [f32; 3]) -> Self {
        UniformValue::F32_3(value)
    }
}
impl<'a> From<&'a [[f32; 3]]> for UniformValue<'a> {
    fn from(value: &'a [[f32; 3]]) -> Self {
        UniformValue::F32_3Array(value)
    }
}
impl<'a> From<&'a [f32; 4]> for UniformValue<'a> {
    fn from(value: &'a [f32; 4]) -> Self {
        UniformValue::F32_4(value)
    }
}
impl<'a> From<&'a [[f32; 4]]> for UniformValue<'a> {
    fn from(value: &'a [[f32; 4]]) -> Self {
        UniformValue::F32_4Array(value)
    }
}
/*
impl<'a> From<&'a Vec4<f32>> for UniformValue<'a> {
    fn from(value: &'a Vec4<f32>) -> Self {
        UniformValue::F32_4(value)
    }
}
impl<'a> From<&'a [[f32; 4]]> for UniformValue<'a> {
    fn from(value: &'a [[f32; 4]]) -> Self {
        UniformValue::F32_4Array(value)
    }
}
*/

struct Uniform<'a> {
    location: &'a WebGLUniformLocation,
    value: UniformValue<'a>,
}

struct Material<'a, I: Iterator<Item = Uniform<'a>>> {
    shader_program: usize,
    uniforms: I,
}

#[wasm_bindgen(start)]
fn main() {
    panic::set_hook(Box::new(|info| eprintln(info.to_string().as_str())));

    REFRESH_CANVAS_SIZE
        .with(|refresh_canvas_size| WINDOW.add_event_listener("resize", refresh_canvas_size));
    refresh_canvas_size();

    GL.with(|gl| {
        gl.clear_color(1.0, 0.0, 0.0, 1.0);
        gl.clear(gl.color_buffer_bit())
    });
    let ShaderProgramIdenitier {
        shader_program_id,
        vertex_shader_id,
        fragment_shader_id,
    } = unsafe {
        SHADER_PROGRAMS.new_program_source_source(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE)
    };
}
