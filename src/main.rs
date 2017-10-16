extern crate glutin;
extern crate gl;
extern crate libc;
extern crate zmq;
extern crate protobuf;
extern crate mzmr_proto;
extern crate rand;
extern crate time;
extern crate prost;

//use mzmr_proto::*;
use mzmr_proto::socket::*;
use mzmr_proto::cmd::*;
use mzmr_proto::cmd::command::*;
use mzmr_proto::cmd::set_resource::*;
use glutin::GlContext;
use rand::Rng;

use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/default.shader"));

mod gl_util;
use gl_util::*;



unsafe fn build_and_use_program() -> ShaderProgram {
	let prog = ShaderProgram::build_program(default::shader);
    prog.bind();
    prog
}

fn set_buffer(table : &mut HashMap<String, HashMap<String,AttributeBuffer>>, name : &String, namespace : &String, dim : u32, data : &[f32]) {
    let buf = table.entry(namespace.clone()).or_insert_with(|| HashMap::new()).entry(name.clone()).or_insert_with(|| AttributeBuffer::new(dim));
    assert!(buf.get_dim() == (dim as usize));
    buf.fill(data);
}

fn set_indices(table : &mut HashMap<String, HashMap<String,IndexBuffer>>, name:&String, namespace:&String, data : &[u32]) {
    let buf = table.entry(namespace.clone()).or_insert_with(|| HashMap::new()).entry(name.clone()).or_insert_with(|| IndexBuffer::new());
    buf.fill(gl::TRIANGLES, data);
}

fn main() {
    let mut event_loop = glutin::EventsLoop::new();

    let window = glutin::WindowBuilder::new()
        .with_title("eyes")
        .with_dimensions(1024,1024);

    let context = glutin::ContextBuilder::new()
        .with_gl_profile(glutin::GlProfile::Core)
        .with_vsync(true);

    let gl_window = glutin::GlWindow::new(window, context, &event_loop).unwrap();

    unsafe {
    	let _ = gl_window.make_current();
    }
    init_gl(|symbol| gl_window.get_proc_address(symbol) as *const _);

    let prog = unsafe {
    	build_and_use_program()
    };

    let instance_id : u64 = rand::thread_rng().gen();
    let instance_name = format!("@{:016X}", instance_id);
    //zmq
    let ctx = Context::new();

    let mut command_socket : XSub<Command,Command> = ctx.xsub().unwrap();
    println!("created");
    command_socket.subscribe(b"render/").unwrap(); //subscribe for all
    println!("render");
    command_socket.subscribe(instance_name.as_bytes()).unwrap(); //subscribe for direct
    println!("self");
    command_socket.connect("tcp://127.0.0.1:1234").unwrap();
    println!("connected", );

    //running data:
    let mut attribute_buffers : HashMap<String, HashMap<String,AttributeBuffer>> = HashMap::new();
    let mut index_buffers : HashMap<String, HashMap<String,IndexBuffer>> = HashMap::new();

    let mut running = true;

    let mut msg_buf : Vec<u8> = Vec::new();
    msg_buf.resize(2048,0);

    unsafe {
        gl::ClearColor(0.0,0.0,0.0,1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
    gl_window.swap_buffers().unwrap();

    let mut current_frame : Option<u32> = None;

    while running {
        unsafe {
            gl::ClearColor(0.0,0.0,0.0,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }


        loop {  
            event_loop.poll_events(|event|{
                match event {
                    glutin::Event::WindowEvent { event:glutin::WindowEvent::Closed, ..} => { running = false; },
                    _ => ()
                }
            });

            if !running {
                break;
            }

            if command_socket.can_recv().unwrap() {
                let (prefix,cmd) = command_socket.recv(&mut msg_buf[..]).unwrap();
                println!("prefix {:?}", std::str::from_utf8(prefix));
                println!("{:?}", cmd);
                match cmd.cmd_data {
                    None => {
                    },
                    Some(CmdData::Done(Done { frame_number, ..})) => { 
                        //unsub from prev frame
                        if let Some(frame) = current_frame {
                            command_socket.unsubscribe(format!("render/frame/{}/",frame).as_bytes()).unwrap();
                        }

                        //subscribe to new frame
                        command_socket.subscribe(format!("render/frame/{}/",frame_number).as_bytes()).unwrap();
                        current_frame = Some(frame_number);

                        //go to next frame
                        break; 
                    },
                    Some(CmdData::SetResource(SetResource { name, namespace, resource, .. })) => {
                        match resource {
                            None => (),
                            Some(Resource::Buffer(Buffer { dim, data, ..})) => {
                                set_buffer(&mut attribute_buffers, &name, &namespace, dim, &data[..]);
                            },
                            Some(Resource::Indices(Indices { data })) => {
                                set_indices(&mut index_buffers, &name, &namespace, &data[..]);
                            }
                            _ => (),
                        }
                    },
                    Some(CmdData::Draw(Draw { attribute_namespaces, .. })) => {
                        let mut draw_call = DrawCall::new();

                        //TODO: get program from data

                        for namespace in &attribute_namespaces {
                            if let Some(some_namespace) = index_buffers.get(namespace) {
                                if let Some(index_buffer) = some_namespace.get("indices") {
                                    draw_call.set_indices(index_buffer);
                                    break;
                                }
                            }
                        }

                        draw_call.set_program(&prog);
                        for &ShaderAttributeInfo { ref name, ..} in &prog.inputs {
                            for namespace in &attribute_namespaces {
                                if let Some(some_namespace) = attribute_buffers.get(namespace) {
                                    if let Some(attrs) = some_namespace.get(name) {
                                        draw_call.add_attrs(name.as_str(), attrs);
                                        break;
                                    }
                                }
                            }
                        }
                
                        draw_call.draw();
                            
                    }
                }
            }
        }

        //show new things, wait for VBLANK
        gl_window.swap_buffers().unwrap();
    }
}
