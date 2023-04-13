use std::ffi::CString;

extern crate sdl2; //Mount sdl2 crate at our crate root module
pub mod render_gl;

fn main() {
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
    let gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    unsafe {
        gl::Viewport(0, 0, 900, 700); //Set the viewport to the size of the window
        gl::ClearColor(0.3, 0.3, 0.5, 1.0); //Set the clear color to a nice blue
    }

    let vert_shader = render_gl::Shader::from_vert_source(
        &CString::new(include_str!("triangle.vert")).unwrap()
    ).unwrap();

    let frag_shader = render_gl::Shader::from_frag_source(
        &CString::new(include_str!("triangle.frag")).unwrap()
    ).unwrap();

    let shader_program = render_gl::Program::from_shaders(
        &[vert_shader, frag_shader]
    ).unwrap();

    shader_program.set_used();

    let verticies: Vec<f32> = vec![
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
        0.0, 0.5, 0.0
    ];
    
    //Get one buffer name and write it into vbo
    let mut vbo: gl::types::GLuint = 0;

    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER, //target
            (verticies.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            verticies.as_ptr() as *const gl::types::GLvoid, //pointer to data
            gl::STATIC_DRAW //usage
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0); //Unbind the buffer
    }

    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
            std::ptr::null()
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    let mut event_pump = sdl.event_pump().unwrap(); //Create an event pump. Gets events from the application
    'main: loop {
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
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindVertexArray(vao);
            gl::DrawArrays(
                gl::TRIANGLES, //mode
                0, //starting index in the enabled arrays
                3 //number of indices to be rendered
            );
        }

        window.gl_swap_window(); //Swap the window buffers
    }
}
