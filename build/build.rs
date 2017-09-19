use std::env;
use std::path::PathBuf;
use std::fs; 

mod shader;

fn main() {

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let _ = fs::remove_dir_all(&out);
    fs::create_dir(&out).unwrap();

    shader::build_shaders();

}


