use core::arch::wasm32;
use shader::{Shader, ShaderProgram};
use slab::Slab;
use std::{
    arch::wasm32::v128,
    mem,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    panic,
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
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct f32x2(v128);
#[allow(non_camel_case_types)]
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct f32x3(v128);
#[allow(non_camel_case_types)]
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct f32x4(v128);

impl DivAssign for f32x4 {
    fn div_assign(&mut self, rhs: Self) {
        *self = self.div(rhs)
    }
}

impl Div for f32x4 {
    type Output = f32x4;

    fn div(self, rhs: Self) -> Self::Output {
        f32x4(wasm32::f32x4_div(self.0, rhs.0))
    }
}

impl MulAssign for f32x4 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.mul(rhs)
    }
}

impl Mul for f32x4 {
    type Output = f32x4;

    fn mul(self, rhs: Self) -> Self::Output {
        f32x4(wasm32::f32x4_mul(self.0, rhs.0))
    }
}

impl SubAssign for f32x4 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.sub(rhs)
    }
}

impl Sub for f32x4 {
    type Output = f32x4;

    fn sub(self, rhs: Self) -> Self::Output {
        f32x4(wasm32::f32x4_sub(self.0, rhs.0))
    }
}

impl AddAssign for f32x4 {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs)
    }
}

impl Add for f32x4 {
    type Output = f32x4;

    fn add(self, rhs: Self) -> Self::Output {
        f32x4(wasm32::f32x4_add(self.0, rhs.0))
    }
}
impl f32x4 {
    pub fn floor(&self) -> Self {
        Self(wasm32::f32x4_floor(self.0))
    }
    pub fn ceil(&self) -> Self {
        Self(wasm32::f32x4_ceil(self.0))
    }
    pub fn abs(&self) -> Self {
        Self(wasm32::f32x4_abs(self.0))
    }
    pub const fn new(a: [f32; 4]) -> f32x4 {
        let [a0, a1, a2, a3] = a;
        f32x4(wasm32::f32x4(a0, a1, a2, a3))
    }
    pub fn splat(f32: f32) -> f32x4 {
        f32x4(wasm32::f32x4_splat(f32))
    }
    pub fn f32x3_f32(a: f32x3, b: f32) -> f32x4 {
        let mut a: f32x4 = unsafe { mem::transmute(a) };
        *a.w_mut() = b;
        a
    }
    fn x(&self) -> &f32 {
        let a: &[f32; 4] = unsafe { mem::transmute(self) };
        &a[0]
    }
    fn x_mut(&mut self) -> &mut f32 {
        let a: &mut [f32; 4] = unsafe { mem::transmute(self) };
        &mut a[0]
    }
    fn y(&self) -> &f32 {
        let a: &[f32; 4] = unsafe { mem::transmute(self) };
        &a[1]
    }
    fn y_mut(&mut self) -> &mut f32 {
        let a: &mut [f32; 4] = unsafe { mem::transmute(self) };
        &mut a[1]
    }
    fn z(&self) -> &f32 {
        let a: &[f32; 4] = unsafe { mem::transmute(self) };
        &a[2]
    }
    fn z_mut(&mut self) -> &mut f32 {
        let a: &mut [f32; 4] = unsafe { mem::transmute(self) };
        &mut a[2]
    }
    fn w(&self) -> &f32 {
        let a: &[f32; 4] = unsafe { mem::transmute(self) };
        &a[3]
    }
    fn w_mut(&mut self) -> &mut f32 {
        let a: &mut [f32; 4] = unsafe { mem::transmute(self) };
        &mut a[3]
    }
}

#[allow(non_camel_case_types)]
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct Mat2(f32x4);

#[allow(non_camel_case_types)]
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct Mat4([f32x4; 4]);

impl Mat4 {
    pub fn quat_to_axes(rotation: Quat) -> (f32x4, f32x4, f32x4) {
        let rotation2 = rotation.0 + rotation.0;

        let [x, y, z, w] = rotation.as_ref();

        let x_rotation = f32x4::splat(*x) * rotation2;
        let y_rotation = f32x4::splat(*y) * rotation2;
        let zz = z * rotation2.z();
        let w_rotation = f32x4::splat(*w) * rotation2;

        let x_axis = f32x4::new([
            1.0 - (y_rotation.y() + zz),
            x_rotation.y() + w_rotation.z(),
            x_rotation.z() - w_rotation.y(),
            0.0,
        ]);
        let y_axis = f32x4::new([
            x_rotation.y() - w_rotation.z(),
            1.0 - (x_rotation.x() + zz),
            (y_rotation.z() + w_rotation.x()),
            0.0,
        ]);
        let z_axis = f32x4::new([
            x_rotation.z() + w_rotation.y(),
            y_rotation.z() - w_rotation.x(),
            1.0 - (x_rotation.x() + y_rotation.y()),
            0.0,
        ]);
        (x_axis, y_axis, z_axis)
    }
    pub fn from_rotation_translation(rotation: Quat, translation: f32x3) -> Mat4 {
        let (x_axis, y_axis, z_axis) = Self::quat_to_axes(rotation);
        Self::from_cols(x_axis, y_axis, z_axis, f32x4::f32x3_f32(translation, 1.0))
    }
    pub const fn from_cols(x_axis: f32x4, y_axis: f32x4, z_axis: f32x4, w_axis: f32x4) -> Self {
        Self([x_axis, y_axis, z_axis, w_axis])
    }
}

#[allow(non_camel_case_types)]
#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct Quat(f32x4);

impl AsRef<[f32; 4]> for Quat {
    fn as_ref(&self) -> &[f32; 4] {
        unsafe { mem::transmute(self) }
    }
}

impl Quat {
    fn x(&self) -> &f32 {
        let a: &[f32; 4] = unsafe { mem::transmute(self) };
        &a[0]
    }
    fn x_mut(&mut self) -> &mut f32 {
        let a: &mut [f32; 4] = unsafe { mem::transmute(self) };
        &mut a[0]
    }
    fn y(&self) -> &f32 {
        let a: &[f32; 4] = unsafe { mem::transmute(self) };
        &a[1]
    }
    fn y_mut(&mut self) -> &mut f32 {
        let a: &mut [f32; 4] = unsafe { mem::transmute(self) };
        &mut a[1]
    }
    fn z(&self) -> &f32 {
        let a: &[f32; 4] = unsafe { mem::transmute(self) };
        &a[2]
    }
    fn z_mut(&mut self) -> &mut f32 {
        let a: &mut [f32; 4] = unsafe { mem::transmute(self) };
        &mut a[2]
    }
    fn w(&self) -> &f32 {
        let a: &[f32; 4] = unsafe { mem::transmute(self) };
        &a[3]
    }
    fn w_mut(&mut self) -> &mut f32 {
        let a: &mut [f32; 4] = unsafe { mem::transmute(self) };
        &mut a[3]
    }
}

#[derive(Clone, Copy)]
struct Transform {
    translation: f32x3,
    rotation: Quat,
}

impl Transform {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.translation)
    }
}

mod uniform;

struct Camera {
    transform: Transform,
    projection: Mat4,
}

trait Material {
    fn get_uniforms<
        const I32_N: usize,
        I32: AsRef<uniform::I32Uniform<I32_N>>,
        I32Fn: Fn(I32),
        const F32_N: usize,
        F32: AsRef<uniform::F32Uniform<F32_N>>,
        F32Fn: Fn(F32),
        const F32X2_N: usize,
        F32x2: AsRef<uniform::F32x2Uniform<F32X2_N>>,
        F32x2Fn: Fn(F32x2),
        const F32X3_N: usize,
        F32x3: AsRef<uniform::F32x3Uniform<F32X3_N>>,
        F32x3Fn: Fn(F32x3),
        const F32X4_N: usize,
        F32x4: AsRef<uniform::F32x4Uniform<F32X4_N>>,
        F32x4Fn: Fn(F32x4),
    >(
        &self,
        i32: I32Fn,
        f32: F32Fn,
        f32x2: F32x2Fn,
        f32x3: F32x3Fn,
        f32x4: F32x4Fn,
    );
}

struct SquareMaterial {
    color: f32,
}

#[wasm_bindgen(start)]
fn main() {
    panic::set_hook(Box::new(|info| eprintln(info.to_string().as_str())));
    let closure = Closure::new(refresh_canvas_size);
    WINDOW.add_event_listener("resize", &closure);
    closure.forget();
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
