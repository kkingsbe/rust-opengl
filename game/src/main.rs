#[macro_use] extern crate failure;

extern crate sdl2; //Mount sdl2 crate at our crate root module
extern crate gl;
pub mod render_gl;
pub mod resources;

use resources::Resources;
use std::path::Path;
use render_gl::data;

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    pos: data::f32_f32_f32,
    clr: data::f32_f32_f32
}

impl Vertex {
    fn vertex_attrib_pointers(gl: &gl::Gl) {
        let stride = std::mem::size_of::<Self>();
        let location = 0;
        let offset = 0;

        unsafe {
            data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
        }

        let location = 1;
        let offset = offset + std::mem::size_of::<data::f32_f32_f32>();

        unsafe {
            data::f32_f32_f32::vertex_attrib_pointer(gl, stride, location, offset);
        }
    }
}

fn main() {
    if let Err(e) = run() {
        println!("{}", failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_relative_exe_path(Path::new("assets-07")).unwrap();
    let sdl = sdl2::init().unwrap(); //Initialize sdl2
    let video_subsystem = sdl.video().unwrap(); //Initialize video subsystem

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5); //Set OpenGL version to 4.5

    let window = video_subsystem
        .window("Game", 900, 700)
        .opengl() //Enable OpenGL
        .resizable()
        .build()
        .unwrap(); //Create a window

    let gl_context = window.gl_create_context().unwrap(); //Create an OpenGL context
    let gl = gl::Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    unsafe {
        gl.Viewport(0, 0, 900, 700); //Set the viewport to the size of the window
        gl.ClearColor(0.3, 0.3, 0.5, 1.0); //Set the clear color to a nice blue
    }
    
    let shader_program = render_gl::Program::from_res(
        &gl, &res, "shaders/triangle"
    ).unwrap();

    shader_program.set_used();

    let verticies: Vec<Vertex> = vec![
        Vertex { pos: (0.5, -0.5, 0.0).into(),  clr: (1.0, 0.0, 0.0).into() },
        Vertex { pos: (-0.5, -0.5, 0.0).into(), clr: (0.0, 1.0, 0.0).into() } ,
        Vertex { pos: (0.0, 0.5, 0.0).into(),   clr: (0.0, 0.0, 1.0).into() }
    ];
    
    //Get one buffer name and write it into vbo
    let mut vbo: gl::types::GLuint = 0;

    unsafe {
        gl.GenBuffers(1, &mut vbo);
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER, //target
            (verticies.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr,
            verticies.as_ptr() as *const gl::types::GLvoid, //pointer to data
            gl::STATIC_DRAW //usage
        );
        gl.BindBuffer(gl::ARRAY_BUFFER, 0); //Unbind the buffer
    }

    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl.GenVertexArrays(1, &mut vao);
        gl.BindVertexArray(vao);
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        Vertex::vertex_attrib_pointers(&gl);
        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        gl.BindVertexArray(0);
    }

    let mut event_pump = sdl.event_pump().unwrap(); //Create an event pump. Gets events from the application
    Ok('main: loop {
        for event in event_pump.poll_iter() {
            //handle user input

            match event {
                sdl2::event::Event::Quit {..} => break 'main,
                _ => {}
            }
        }

        //render window contents
        shader_program.set_used();
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
            gl.BindVertexArray(vao);
            gl.DrawArrays(
                gl::TRIANGLES, //mode
                0, //starting index in the enabled arrays
                3 //number of indices to be rendered
            );
        }

        window.gl_swap_window(); //Swap the window buffers
    })
}

pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e.iter_chain().collect::<Vec<_>>().into_iter().rev().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut result, "    Which caused the following issue:");
        }

        let _ = write!(&mut result, "{}", cause);

        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}