mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::*;
//pub use bindings::Gl as InnerGl;

use std::{rc::Rc, ops::Deref};
//A wrapper struct for the gl crate which uses a reference counter
#[derive(Clone)]
pub struct Gl {
    inner: Rc<bindings::Gl>
}

impl Gl {
    pub fn load_with<F>(loadfn: F) -> Gl
        where F: FnMut(&'static str) -> *const types::GLvoid
    {
        Gl {
            inner: Rc::new(bindings::Gl::load_with(loadfn))
        }
    }
}

//Forward all calls to the "inner" item in the struct
impl Deref for Gl {
    type Target = bindings::Gl;

    fn deref(&self) -> &bindings::Gl {
        &self.inner
    }
}