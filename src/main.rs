use crate::{helper::Helper::CLI, model::model::Pipe};
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::FileTypeExt;
use std::sync::{Arc, Mutex};
use std::thread;
use nix::unistd::mkfifo;
use nix::sys::stat::Mode;

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

    if clargs.dbg{
        println!("{clargs:?}");
    }
    
    let mut pipe;
    if &clargs.ipipe != &clargs.opipe{
        pipe = Pipe::new(false);

    }else{
        pipe = Pipe::new(true);
    }

    let ip = clargs.ipipe.as_ref().unwrap();
    let op = clargs.opipe.as_ref().unwrap();
    init_fifo(ip);
    if ip != op {
        init_fifo(op);
    }

    let pipe = Arc::new(Mutex::new(pipe));
    let ip_clone = ip.to_string();
    let op_clone = op.to_string();
    let pipe_clone_ip = Arc::clone(&pipe);
    let pipe_clone_op = Arc::clone(&pipe);

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
    let mut buffer = [0u8; 4096];

    loop {
        thread::sleep(std::time::Duration::from_millis(10));        
        let (rx_data, tx_data) = pipe.lock().unwrap().peek(None);
        
        if let Some(data) = rx_data {
            // TODO: Add processing logic here for data from input pipe
            // Process the data as needed before writing to output pipe
            let args = data.split(" ").collect::<Vec<&str>>();

            if data.len() == 0{
                break;
            }

            perform(args);

            if let Err(e) = write_file.write_all(data.as_bytes()) {
                eprintln!("Failed to write to output FIFO: {}", e);
                break;
            }            
        }
        
        if let Some(data) = tx_data {
            // TODO: Add processing logic here for data from output pipe
            // Process the data as needed before writing to input pipe
            
            let mut input_write = fs::OpenOptions::new().write(true).open(ip).expect("Failed to open input FIFO for writing");
            if let Err(e) = input_write.write_all(data.as_bytes()) {panic!("Failed to write to input FIFO: {}", e);}
        }
    }

    ip_thread.join().unwrap();
    op_thread.join().unwrap();

}
