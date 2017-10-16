
extern crate mzmr_proto;

use mzmr_proto::*;
use mzmr_proto::socket::*;
use mzmr_proto::cmd::*;
use mzmr_proto::cmd::command::*;
use mzmr_proto::cmd::set_resource::*;

fn handle_register<'a,'b>(topic : &mut Topic<'a,'b,Command>, frame : u32, cmds : &[Command]) -> Result<()> {
    for cmd in cmds {
        topic.send(cmd)?;
    }
    topic.send(&Command { cmd_data : Some(CmdData::Done(Done{ frame_number : frame }))})?;
    Ok(())
}

fn main() {
	let ctx = Context::new();
	let mut socket : XPub<Command,Command> = ctx.xpub().unwrap();
	socket.bind("tcp://127.0.0.1:1234").unwrap();


    let pos_buf = Buffer {
        dim : 2,
        data : vec![
            -0.5, -0.5,
            -0.5, 0.5,
            0.5, -0.5,
            0.5, 0.5
        ],
    };

    let col_buf = Buffer {
        dim : 3,
        data : vec![
            1.0, 1.0, 0.0,
            0.0, 1.0, 1.0,
            1.0, 0.0, 1.0,
            1.0, 1.0, 1.0,
        ],
    };
    

	let index_buf = Indices {
        data : vec![
            0,1,2,
            0,2,3,
        ],
    };


    let pos_cmd = Command { cmd_data : Some(CmdData::SetResource(SetResource { 
        name:"position".to_string(), 
        namespace:"tri".to_string(), 
        resource: Some(Resource::Buffer(pos_buf))
    }))};

    let col_cmd = Command { cmd_data : Some(CmdData::SetResource(SetResource { 
        name:"color".to_string(), 
        namespace:"tri".to_string(), 
        resource: Some(Resource::Buffer(col_buf))
    }))};

    let idx_cmd = Command { cmd_data : Some(CmdData::SetResource(SetResource { 
        name:"indices".to_string(), 
        namespace:"tri".to_string(), 
        resource: Some(Resource::Indices(index_buf))
    }))};

    let init_cmds = [pos_cmd, col_cmd, idx_cmd];

    let draw = Draw {
        program_name : "".to_string(),
        program_namespace : "".to_string(),
        attribute_namespaces : vec!["tri".to_string()],
        uniform_namespaces : Vec::new(),
    };
    let mut recv_buffer : Vec<u8> = Vec::new();
    recv_buffer.resize(2048,0);

    for frame in 1.. {

        loop {

            match socket.recv(&mut recv_buffer[..]).unwrap() {
                SubscriptionInfo::Unsubscribe(topic) => {
                    println!("unsubscribe: {:?}", topic);
                },
                SubscriptionInfo::Message(_) => {
                    println!("message?");
                },
                SubscriptionInfo::Subscribe(mut subscription) => {
                    println!("subscribe: {:?}", subscription.topic);
                    if &subscription.topic[0..6] != b"render" {
                        //send data on subscription
                        handle_register(&mut subscription, frame, &init_cmds[..]).unwrap();
                    }

                    //new frame
                    let match_target = frame.to_string() + "/";
                    if subscription.topic.ends_with(match_target.as_bytes()) {
                        break;
                    }
                },
            }
        }

        let mut topic = socket.topic(b"render/");
        topic.send(&Command { cmd_data : Some(CmdData::Draw(draw.clone()))}).unwrap();
        topic.send(&Command { cmd_data : Some(CmdData::Done(Done{ frame_number : (frame+1) }))}).unwrap();
        println!("next frame: {:?}", frame+1);

    }
}
