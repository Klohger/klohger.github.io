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

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy)]
struct i32x2(v128);
#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy)]
struct i32x3(v128);
#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy)]
struct i32x4(v128);

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy)]
struct f32x2(v128);
#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy)]
struct f32x3(v128);
#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy)]
struct f32x4(v128);

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy)]
struct Mat2(v128);

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy)]
struct Mat4([v128; 4]);

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy)]
struct Quat(v128);

#[derive(Clone, Copy)]
struct Transform {
    translation: f32x3,
    rotation: Quat,
}

#[repr(transparent)]
struct I32Uniform<const N: usize>([i32; N]);

impl From<i32> for I32Uniform<1> {
    fn from(value: i32) -> I32Uniform<1> {
        I32Uniform([value])
    }
}
impl<const N: usize> From<[i32; N]> for I32Uniform<N> {
    fn from(value: [i32; N]) -> I32Uniform<N> {
        I32Uniform(value)
    }
}

#[repr(transparent)]
struct I32x2Uniform<const N: usize>([[i32; 2]; N]);

impl From<[i32; 2]> for I32x2Uniform<1> {
    fn from(value: [i32; 2]) -> I32x2Uniform<1> {
        I32x2Uniform([value])
    }
}
impl<const N: usize> From<[[i32; 2]; N]> for I32x2Uniform<N> {
    fn from(value: [[i32; 2]; N]) -> I32x2Uniform<N> {
        I32x2Uniform(value)
    }
}
impl From<i32x2> for I32x2Uniform<1> {
    fn from(value: i32x2) -> I32x2Uniform<1> {
        I32x2Uniform([unsafe { *(&value as *const _ as *const [i32; 2]) }])
    }
}

#[repr(transparent)]
struct I32x3Uniform<const N: usize>([[i32; 3]; N]);

impl From<[i32; 3]> for I32x3Uniform<1> {
    fn from(value: [i32; 3]) -> I32x3Uniform<1> {
        I32x3Uniform([value])
    }
}
impl<const N: usize> From<[[i32; 3]; N]> for I32x3Uniform<N> {
    fn from(value: [[i32; 3]; N]) -> I32x3Uniform<N> {
        I32x3Uniform(value)
    }
}
impl From<i32x3> for I32x3Uniform<1> {
    fn from(value: i32x3) -> I32x3Uniform<1> {
        I32x3Uniform([unsafe { *(&value as *const _ as *const [i32; 3]) }])
    }
}

#[repr(transparent)]
struct I32x4Uniform<const N: usize>([[i32; 4]; N]);

impl From<[i32; 4]> for I32x4Uniform<1> {
    fn from(value: [i32; 4]) -> I32x4Uniform<1> {
        I32x4Uniform([value])
    }
}
impl<const N: usize> From<[[i32; 4]; N]> for I32x4Uniform<N> {
    fn from(value: [[i32; 4]; N]) -> I32x4Uniform<N> {
        I32x4Uniform(value)
    }
}
impl From<i32x2> for I32x4Uniform<1> {
    fn from(value: i32x2) -> I32x4Uniform<1> {
        I32x4Uniform([unsafe { mem::transmute(value) }])
    }
}
impl<const N: usize> From<[i32x2; N]> for I32x4Uniform<N> {
    fn from(value: [i32x2; N]) -> I32x4Uniform<N> {
        I32x4Uniform(unsafe { mem::transmute(value) })
    }
}
impl From<i32x3> for I32x4Uniform<1> {
    fn from(value: i32x3) -> I32x4Uniform<1> {
        I32x4Uniform([unsafe { mem::transmute(value) }])
    }
}
impl<const N: usize> From<[i32x3; N]> for I32x4Uniform<N> {
    fn from(value: [i32x3; N]) -> I32Uniform<1> {
        I32x4Uniform(unsafe { mem::transmute(value) })
    }
}
impl From<i32x4> for I32x4Uniform<1> {
    fn from(value: i32x4) -> I32Uniform<1> {
        I32x4Uniform([unsafe { mem::transmute(value) }])
    }
}
impl<const N: usize> From<[i32x4; N]> for I32x4Uniform<N> {
    fn from(value: [i32x4; N]) -> I32Uniform<1> {
        I32x4Uniform(unsafe { mem::transmute(value) })
    }
}

#[repr(transparent)]
struct F32Uniform<const N: usize>([f32; N]);

impl From<f32> for F32Uniform<1> {
    fn from(value: f32) -> I32Uniform<1> {
        F32Uniform([value])
    }
}
impl<const N: usize> From<[f32; N]> for F32Uniform<N> {
    fn from(value: [f32; N]) -> I32Uniform<1> {
        F32Uniform(value)
    }
}

#[repr(transparent)]
struct F32x2Uniform<const N: usize>([[f32; 2]; N]);

impl From<[f32; 2]> for F32x2Uniform<1> {
    fn from(value: [f32; 2]) -> I32Uniform<1> {
        F32x2Uniform([value])
    }
}
impl<const N: usize> From<[[f32; 2]; N]> for F32x2Uniform<N> {
    fn from(value: [[f32; 2]; N]) -> I32Uniform<1> {
        F32x2Uniform(value)
    }
}
impl From<f32x2> for F32x2Uniform<1> {
    fn from(value: f32x2) -> I32Uniform<1> {
        F32x2Uniform([unsafe { *(&value as *const _ as *const [f32; 2]) }])
    }
}

#[repr(transparent)]
struct F32x3Uniform<const N: usize>([[f32; 3]; N]);

impl From<[f32; 3]> for F32x3Uniform<1> {
    fn from(value: [f32; 3]) -> I32Uniform<1> {
        F32x3Uniform([value])
    }
}
impl<const N: usize> From<[[f32; 3]; N]> for F32x3Uniform<N> {
    fn from(value: [[f32; 3]; N]) -> I32Uniform<1> {
        F32x3Uniform(value)
    }
}
impl From<f32x3> for F32x3Uniform<1> {
    fn from(value: f32x3) -> I32Uniform<1> {
        F32x3Uniform([unsafe { *(&value as *const _ as *const [f32; 3]) }])
    }
}

#[repr(transparent)]
struct F32x4Uniform<const N: usize>([[f32; 4]; N]);

impl From<[f32; 4]> for F32x4Uniform<1> {
    fn from(value: [f32; 4]) -> I32Uniform<1> {
        F32x4Uniform([value])
    }
}
impl<const N: usize> From<[[f32; 4]; N]> for F32x4Uniform<N> {
    fn from(value: [[f32; 4]; N]) -> I32Uniform<1> {
        F32x4Uniform(value)
    }
}
impl From<f32x2> for F32x4Uniform<1> {
    fn from(value: f32x2) -> I32Uniform<1> {
        F32x4Uniform([unsafe { mem::transmute(value) }])
    }
}
impl<const N: usize> From<[f32x2; N]> for F32x4Uniform<N> {
    fn from(value: [f32x2; N]) -> I32Uniform<1> {
        F32x4Uniform(unsafe { mem::transmute(value) })
    }
}
impl From<f32x3> for F32x4Uniform<1> {
    fn from(value: f32x3) -> I32Uniform<1> {
        F32x4Uniform([unsafe { mem::transmute(value) }])
    }
}
impl<const N: usize> From<[f32x3; N]> for F32x4Uniform<N> {
    fn from(value: [f32x3; N]) -> I32Uniform<1> {
        F32x4Uniform(unsafe { mem::transmute(value) })
    }
}
impl From<f32x4> for F32x4Uniform<1> {
    fn from(value: f32x4) -> I32Uniform<1> {
        F32x4Uniform([unsafe { mem::transmute(value) }])
    }
}
impl<const N: usize> From<[f32x4; N]> for F32x4Uniform<N> {
    fn from(value: [f32x4; N]) -> I32Uniform<1> {
        F32x4Uniform(unsafe { mem::transmute(value) })
    }
}

trait Material {
    fn get_uniforms<
        const I_N: usize,
        I: Into<I32Uniform<I_N>>,
        I_Fn: Fn(I),
        const Ix2_N: usize,
        Ix2: Into<I32x2Uniform<Ix2_N>>,
        Ix2_Fn: Fn(Ix2),
        const Ix3_N: usize,
        Ix3: Into<I32x3Uniform<Ix3_N>>,
        Ix3_Fn: Fn(Ix3),
        const Ix4_N: usize,
        Ix4: Into<I32x4Uniform<Ix4_N>>,
        Ix4_Fn: Fn(Ix4),
        const F_N: usize,
        F: Into<F32Uniform<F_N>>,
        F_Fn: Fn(F),
        const Fx2_N: usize,
        Fx2: Into<F32x2Uniform<Fx2_N>>,
        Fx2_Fn: Fn(Fx2),
        const Fx3_N: usize,
        Fx3: Into<F32x3Uniform<Fx3_N>>,
        Fx3_Fn: Fn(Fx3),
        const Fx4_N: usize,
        Fx4: Into<F32x4Uniform<Fx4_N>>,
        Fx4_Fn: Fn(Fx4),
    >(
        &self,
        i32: I_Fn,
        i32x2: Ix2_Fn,
        i32x3: Ix3_Fn,
        i32x4: Ix4_Fn,
        f32: F_Fn,
        f32x2: Fx2_Fn,
        f32x3: Fx3_Fn,
        f32x4: Fx4_Fn,
    );
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
