use std::ffi::{CStr, CString};
use gl; //Mount gl crate at our crate root module
use resources::Resources;

use crate::resources;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to load resource {}", name)]
    ResourceLoad {
        name: String,
        #[cause] inner: resources::Error
    },
    #[fail(display = "Can not determine shader type for resource {}", name)]
    CanNotDetermineShaderTypeForResource {
        name: String
    },
    #[fail(display = "Failed to compile shader {}", name)]
    CompileError {
        name: String,
        message: String
    },
    #[fail(display = "Failed to link program {}", name)]
    LinkError {
        name: String,
        message: String
    }
}

pub struct Program {
    gl: gl::Gl,
    id: gl::types::GLuint
}

impl Program {
    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, Error> {
        let program_id = unsafe {
            gl.CreateProgram()
        };

        for shader in shaders {
            unsafe {
                gl.AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl.LinkProgram(program_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl.GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                )
            }

            return Err(Error::CompileError { name: "CompileError".to_string(), message: error.to_string_lossy().into_owned() });
        }

        for shader in shaders {
            unsafe {
                gl.DetachShader(program_id, shader.id());
            }
        }

        Ok(Program { gl: gl.clone(), id: program_id })
    }

    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Program, Error> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag"
        ];

        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_res(gl, res, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<Shader>, Error>>()?;

        Program::from_shaders(gl, &shaders[..])
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    gl: gl::Gl,
    id: gl::types::GLuint
}

impl Shader {
    //Loads a shader from the given string. Acts as a constructor for the Shader struct
    pub fn from_source(gl: &gl::Gl, source: &CStr, kind: gl::types::GLenum) -> Result<Shader, Error> {
        let id = shader_from_source(gl, source, kind)?;
        Ok(Shader { gl: gl.clone(), id })
    }

    pub fn from_vert_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, Error> {
        Shader::from_source(gl, source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, Error> {
        Shader::from_source(gl, source, gl::FRAGMENT_SHADER)
    }

    pub fn from_res(gl: &gl::Gl, res: &Resources, name: &str) -> Result<Shader, Error> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER)
        ];

        //Check file extension to determine type of shader
        let shader_kind = POSSIBLE_EXT.iter()
            .find(|&&(file_extension, _)| {
                name.ends_with(file_extension)
            })
            .map(|&(_, kind)| kind)
            .ok_or_else(|| Error::CanNotDetermineShaderTypeForResource { name: format!("Cannot determine shader type for resource{}", name)})?;

        let source = res.load_cstring(name)
            .map_err(|e| Error::ResourceLoad { name: format!("Error loading resource {}: {:?}", name, e), inner: e })?;

        Shader::from_source(gl, &source, shader_kind)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}   

//This trait is executed when the Shader struct is about to be deallocated
impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id); //Remove the shader. Not automatically done.
        }
    }
}

fn shader_from_source(gl: &gl::Gl, source: &CStr, kind: gl::types::GLuint) -> Result<gl::types::GLuint, Error> {
    //Get shader object id
    let id = unsafe {
        gl.CreateShader(kind)
    };

    unsafe {
        gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    };

    let mut success: gl::types::GLint = 1;
    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    };

    //Compilation error
    if success == 0 {
        //Get error message
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar //A pointer to the string buffer that the error message should be written into
            );
        }

        return Err(Error::CompileError { name: "CompileError".to_string(), message: error.to_string_lossy().into_owned()});
    }

    Ok(id)
}

//Creates a null-terminated C string filled with whitespace wiht length len
fn create_whitespace_cstring_with_len(len: usize) -> CString {
    //Allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);

    //Fill it with spaces (how is there not a better way to do this lol)
    buffer.extend([b' '] //Puts a space in the buffer
          .iter() //Converts the space to an iterator
          .cycle() //Makes the iterator infinite
          .take(len as usize)); //Takes the first len elements

    //Convert buffer to CString
    unsafe {
        CString::from_vec_unchecked(buffer)
    }
}