use std::{
    ops::Deref,
    ptr::{null, null_mut},
};

use gl::types::GLfloat;
use glutin::prelude::GlDisplay;
use std::ffi::{CStr, CString};

use glm;

use crate::{camera::Camera, light::SunLight};

pub mod gl {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

    pub use Gles2 as Gl;
}

pub struct Renderer {
    program: gl::types::GLuint,
    vao: gl::types::GLuint,
    vbo: gl::types::GLuint,
    ray_bo: gl::types::GLuint,
    uniform_bo: gl::types::GLuint,
    gl: gl::Gl,
}

#[repr(C)]
#[derive(Debug)]
pub struct UniformData {
    origin: glm::Vec3,
    _0: i32,
    light_dir: glm::Vec3,
    _1: i32,
    light_color: glm::Vec3,
    stop_distance: f32,
}

impl Renderer {
    pub fn new<D: GlDisplay>(gl_display: &D) -> Self {
        println!("Creating OpenGL stuff...");
        unsafe {
            let gl = gl::Gl::load_with(|symbol| {
                let symbol = CString::new(symbol).unwrap();
                gl_display.get_proc_address(symbol.as_c_str()).cast()
            });

            if let Some(renderer) = get_gl_string(&gl, gl::RENDERER) {
                println!("Running on {}", renderer.to_string_lossy());
            }
            if let Some(version) = get_gl_string(&gl, gl::VERSION) {
                println!("OpenGL Version {}", version.to_string_lossy());
            }

            if let Some(shaders_version) = get_gl_string(&gl, gl::SHADING_LANGUAGE_VERSION) {
                println!("Shaders version on {}", shaders_version.to_string_lossy());
            }

            // Compile shader program

            let vertex_shader = create_shader(&gl, gl::VERTEX_SHADER, VERTEX_SHADER_SOURCE);
            let fragment_shader = create_shader(
                &gl,
                gl::FRAGMENT_SHADER,
                FRAGMENT_SHADER_SOURCE.to_bytes_with_nul(),
            );

            let program = gl.CreateProgram();

            gl.AttachShader(program, vertex_shader);
            gl.AttachShader(program, fragment_shader);

            gl.LinkProgram(program);

            gl.UseProgram(program);

            gl.DeleteShader(vertex_shader);
            gl.DeleteShader(fragment_shader);

            println!("Compiled shaders.");

            let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
            let ray_attrib = gl.GetAttribLocation(program, b"ray\0".as_ptr() as *const _);
            let uniform_attrib = gl.GetUniformBlockIndex(program, b"uni\0".as_ptr() as *const _);
            gl.UniformBlockBinding(program, uniform_attrib, 0);

            // This is for vertex indices
            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);

            // Create constant vertex data buffer

            let mut vbo = std::mem::zeroed();
            gl.GenBuffers(1, &mut vbo);
            gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                VERTEX_DATA.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl.VertexAttribPointer(
                pos_attrib as gl::types::GLuint,
                2,
                gl::FLOAT,
                0,
                0,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);

            // Create ray direction buffer
            let mut ray_bo = std::mem::zeroed();
            gl.GenBuffers(1, &mut ray_bo);
            gl.BindBuffer(gl::ARRAY_BUFFER, ray_bo);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (4 * std::mem::size_of::<glm::Vec3>()) as gl::types::GLsizeiptr,
                null(),
                gl::DYNAMIC_DRAW,
            );
            gl.VertexAttribPointer(
                ray_attrib as gl::types::GLuint,
                3,
                gl::FLOAT,
                1,
                0,
                std::ptr::null(),
            );
            gl.EnableVertexAttribArray(ray_attrib as gl::types::GLuint);

            // Create uniform buffer
            let mut uniform_bo = std::mem::zeroed();
            gl.GenBuffers(1, &mut uniform_bo);
            gl.BindBuffer(gl::UNIFORM_BUFFER, uniform_bo);
            gl.BufferData(
                gl::UNIFORM_BUFFER,
                std::mem::size_of::<UniformData>() as isize,
                null(),
                gl::DYNAMIC_DRAW,
            );
            gl.BindBufferRange(
                gl::UNIFORM_BUFFER,
                uniform_attrib as gl::types::GLuint,
                uniform_bo,
                0,
                std::mem::size_of::<UniformData>() as isize,
            );

            Self {
                program,
                vao,
                vbo,
                ray_bo,
                uniform_bo,
                gl,
            }
        }
    }

    pub fn draw(&self, camera: &Camera, light: &SunLight) {
        self.draw_with_clear_color(camera, light, 0.1, 0.1, 0.1, 0.9);
    }

    pub fn draw_with_clear_color(
        &self,
        camera: &Camera,
        light: &SunLight,
        red: GLfloat,
        green: GLfloat,
        blue: GLfloat,
        alpha: GLfloat,
    ) {
        let corners = camera.get_corners();
        let uniform_data = [UniformData {
            origin: camera.position,
            light_dir: light.direction,
            light_color: light.color,
            stop_distance: camera.get_stop_distance(),
            _0: 0,
            _1: 0,
        }];

        unsafe {
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.ray_bo);
            self.gl.BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                std::mem::size_of::<[glm::Vec3; 4]>() as isize,
                corners.as_ptr() as *const _,
            );
            self.gl.BindBuffer(gl::UNIFORM_BUFFER, self.uniform_bo);
            self.gl.BufferSubData(
                gl::UNIFORM_BUFFER,
                0,
                std::mem::size_of::<UniformData>() as isize,
                uniform_data.as_ptr() as *const _,
            );
            self.gl.BindBuffer(gl::UNIFORM_BUFFER, 0);
        }

        unsafe {
            self.gl.UseProgram(self.program);

            self.gl.BindVertexArray(self.vao);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            // self.gl.BindBuffer(gl::ARRAY_BUFFER, self.ray_bo);
            // self.gl.BindBufferBase(gl::UNIFORM_BUFFER, 0, self.uniform_bo);

            self.gl.ClearColor(red, green, blue, alpha);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
            self.gl.DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            self.gl.Viewport(0, 0, width, height);
        }
    }
}

impl Deref for Renderer {
    type Target = gl::Gl;

    fn deref(&self) -> &Self::Target {
        &self.gl
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.program);
            self.gl.DeleteBuffers(1, &self.uniform_bo);
            self.gl.DeleteBuffers(1, &self.ray_bo);
            self.gl.DeleteBuffers(1, &self.vbo);
            self.gl.DeleteVertexArrays(1, &self.vao);
        }
    }
}

unsafe fn create_shader(
    gl: &gl::Gl,
    shader: gl::types::GLenum,
    source: &[u8],
) -> gl::types::GLuint {
    let shader = gl.CreateShader(shader);
    gl.ShaderSource(
        shader,
        1,
        [source.as_ptr().cast()].as_ptr(),
        std::ptr::null(),
    );
    gl.CompileShader(shader);

    let mut success = 1;
    gl.GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

    if success == 0 {
        let mut len = 0;
        gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

        let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
        buffer.set_len(len as usize);
        gl.GetShaderInfoLog(shader, len, null_mut(), buffer.as_mut_ptr() as *mut i8);

        println!("{}", String::from_utf8_unchecked(buffer));
    }

    shader
}

fn get_gl_string(gl: &gl::Gl, variant: gl::types::GLenum) -> Option<&'static CStr> {
    unsafe {
        let s = gl.GetString(variant);
        (!s.is_null()).then(|| CStr::from_ptr(s.cast()))
    }
}

#[rustfmt::skip]
static VERTEX_DATA: [f32; 8] = [
    -1.0, -1.0,
    -1.0, 1.0,
    1.0, -1.0,
    1.0, 1.0
];

const VERTEX_SHADER_SOURCE: &[u8] = b"
#version 100
precision mediump float;

attribute vec2 position;
attribute vec3 ray;

varying vec3 ray_direction;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    ray_direction = ray;
}
\0";

const FRAGMENT_SHADER_SOURCE: &CStr = crate::macros::include_cstr!("shader/mandelbulb.glsl");
