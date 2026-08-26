use crate::model::model::{KGraph, perform, generate_sample_graph};
use crate::{helper::Helper::CLI, model::model::Pipe};
use std::env::args_os;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::FileTypeExt;
use std::sync::{Arc, Mutex};
use std::thread;
use nix::unistd::mkfifo;
use nix::sys::stat::Mode;
use uuid::Uuid;

mod helper;
mod model;

fn init_fifo(path: &str) {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.file_type().is_fifo() {
            return;
        }
    } else {
        if let Err(e) = mkfifo(path, Mode::from_bits_truncate(0o666)) {
            eprintln!("Failed to create FIFO {}: {}", path, e);
            std::process::exit(1);
        }
    }
}

fn main() {
    let mut clargs = CLI::new();
    clargs.Parse_Args();
    let mut knowledge = generate_sample_graph();
    if clargs.dbg{
        println!("{clargs:?}");
    }
    let mut pipe = Arc::new(Mutex::new(if &clargs.ipipe != &clargs.opipe{
        Pipe::new(false)
    }else{
        Pipe::new(true)
    }));
    let pipe_clone_ip = Arc::clone(&pipe);
    let pipe_clone_op = Arc::clone(&pipe);

    let ip = clargs.ipipe.as_ref().unwrap();
    let op = clargs.opipe.as_ref().unwrap();
    let ip_clone = ip.to_string();
    let op_clone = op.to_string();

    init_fifo(ip);
    if ip != op {
        init_fifo(op);
    }



    let ip_thread = thread::spawn(move || {
        let mut input_file = fs::OpenOptions::new().read(true).open(&ip_clone).expect("Failed to open input FIFO");
        let mut buffer = [0u8; 4096];
        loop {
            match input_file.read(&mut buffer) {
                Ok(0) => {continue;}
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                    pipe_clone_ip.lock().unwrap().append(data.clone(), Some(crate::model::model::e_Pipe_io::Rx));
                }
                Err(e) => { panic!("Failed to read from input FIFO: {}", e);}
            }
        }
    });

    let op_thread = thread::spawn(move || {
        let mut output_file = fs::OpenOptions::new().read(true).open(&op_clone).expect("Failed to open output FIFO");
        let mut buffer = [0u8; 4096];
        loop {
            match output_file.read(&mut buffer) {
                Ok(0) => {continue;}
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                    pipe_clone_op.lock().unwrap().append(data.clone(), Some(crate::model::model::e_Pipe_io::Tx));
                }
                Err(e) => { panic!("Failed to read from output FIFO: {}", e);}
            }
        }
    });



    let mut write_file = fs::OpenOptions::new().write(true).open(op).expect("Failed to open output FIFO for writing");
    let curp = knowledge.tree.as_ref().unwrap().uuid;
    loop {
        thread::sleep(std::time::Duration::from_millis(10));        
        let (rx_data, tx_data) = pipe.lock().unwrap().peek(None);
        
        if let Some(data) = rx_data {
            // TODO: Add processing logic here for data from input pipe
            // Process the data as needed before writing to output pipe
            let arg = data.trim();
            if arg.len() == 0{
                break;
            }
            let mut full_buff= String::new();
            let queries = arg.split("\n\n").collect::<Vec<&str>>();
            for query in queries{ 
                let mut buff = String::new();
                let args = query.split(" ").collect::<Vec<&str>>();
                if args.len() == 1{
                    let neigh = knowledge.get_neighbors(args[0].parse::<Uuid>().expect("Uuid is a whole number"));
                    for x in &neigh{
                        let nd = knowledge.get_node(*x);
                        if let Some(z) = nd{
                            buff += &String::from_utf8(z.data.to_vec()).unwrap();
                        }else{
                            buff += "Not found";
                        }    
                        buff += "\n\n\n";
                    }
                }else{
                    let cmnd = args[0];
                    match cmnd{
                        "get_node" => {
                            for nd_uid in args.iter().skip(1){
                                let uuid = nd_uid.parse::<Uuid>().expect("Uuid is a unsigned integer");
                                let ret = knowledge.get_node(uuid);
                                if let Some(p) = ret{
                                     buff += &String::from_utf8(p.data.to_vec()).unwrap();
                                } else{
                                    buff += "Not Found";
                                }
                                buff += "\n\n\n";
                            }
                        },
                        "get_neighbour" => {
                            for nd_uid in args.iter().skip(1){
                                let uuid = nd_uid.parse::<Uuid>().expect("Uuid is a unsigned integer");
                                let ret = knowledge.get_neighbors(uuid);
                                for j in &ret{
                                    if let Some(p) = knowledge.get_node(*j){
                                        buff += &String::from_utf8(p.data.to_vec()).unwrap();
                                    } else{
                                        buff += "Not Found";
                                    }
                                    buff += "\n"
                                }
                                buff += "\n\n\n";
                            }
                        },
                        "prune" => {
                            if args.len() < 3{
                                buff += "The args provided to prune subcommand where wrong/incorrectly provided\n\n\n";
                            }else{
                                let (mn,mx);
                                if args.len() == 3{
                                    mx = args[1].parse::<usize>().expect("Maximum is unsigned int");
                                    mn = 0;
                                }else{
                                    mn = args[1].parse::<usize>().expect("Minimum is unsigned int");
                                    mx = args[2].parse::<usize>().expect("Maximum is unsigned int");
                                }
                                for i in args.iter().skip(3){
                                    let uuid = i.parse::<Uuid>().expect("Uuid is a unsigned integer");
                                    let nodes_within_dist = knowledge.find_nodes_within_distance(uuid, mn, mx);
                                    for (node_uuid, dist) in nodes_within_dist {
                                        if let Some(node) = knowledge.get_node(node_uuid) {
                                            buff += &format!("UUID: {}, Distance: {}, Data: {}\n", node_uuid, dist, String::from_utf8(node.data.to_vec()).unwrap_or_else(|_| "Invalid UTF-8".to_string()));
                                        } else {
                                            buff += &format!("UUID: {}, Distance: {}, Data: Not Found\n", node_uuid, dist);
                                        }
                                    }
                                    buff += "\n\n\n";
                                }
                            }

                        }
                         _ => {},
                         
                    }
                }

                full_buff += "----------";
                full_buff += &buff;
            }

            if let Err(e) = write_file.write_all(full_buff.as_bytes()) {
                eprintln!("Failed to write to output FIFO: {}", e);
                break;
            }            
        }
        
        if let Some(data) = tx_data {
            let mut input_write = fs::OpenOptions::new().write(true).open(ip).expect("Failed to open input FIFO for writing");
            if let Err(e) = input_write.write_all(data.as_bytes()) {panic!("Failed to write to input FIFO: {}", e);}
        }
        if !clargs.sess{
            break;
        }
    }

    ip_thread.join().unwrap();
    op_thread.join().unwrap();

}
