use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::io::Read;


pub fn build_shaders() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let shader_source = root.join("shaders");
    build_shader(&shader_source, &out, "default");
}

fn write_shader(src_dir : &PathBuf, f : &mut File, name : &str, suffix : &str) {
	let path = src_dir.join(name).with_extension(suffix);
	let mut file = File::open(path).unwrap();
	let mut text = String::new();
	file.read_to_string(&mut text).unwrap();
	write!(f, "b\"\n").unwrap();
	write!(f, "{}", text).unwrap();
	write!(f, "\\0\"\n").unwrap();

}
fn build_shader(src_dir : &PathBuf, dst_dir : &PathBuf, name : &str) {
	let mod_path = dst_dir.join(name).with_extension("shader");
	let mut f = File::create(mod_path).unwrap();

	write!(f, "mod {} {{\n",name).unwrap();
	write!(f, "#[allow(non_upper_case_globals)]\n").unwrap();
	write!(f, "pub static shader : (&'static [u8], &'static [u8]) = (\n").unwrap();
	write_shader(src_dir, &mut f, name, "vert");
	write!(f, ",\n").unwrap();
	write_shader(src_dir, &mut f, name, "frag");
	write!(f, ");\n").unwrap();
	write!(f, "}}").unwrap();
}