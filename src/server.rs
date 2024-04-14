use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;

use crate::task::Task;

pub trait ServerTrait {
    fn start_server(
        &self,
        address: String,
        tx: mpsc::Sender<Result<(), Box<dyn Error + Send>>>,
    );
}

pub struct Server;

impl ServerTrait for Server {
    fn start_server(
        &self,
        address: String,
        tx: mpsc::Sender<Result<(), Box<dyn Error + Send>>>,
    ) {
        println!("Starting the server");
        let listener = TcpListener::bind(address);

        match listener {
            Ok(_) => tx.send(Ok(())).unwrap(),
            Err(e) => {
                println!("here {}", e);
                tx.send(Err(Box::new(e))).unwrap();
                return;
            }
        }

        for stream in listener.unwrap().incoming() {
            match stream {
                Ok(stream) => {
                    Self::handle_connection(stream);
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

impl Server {
    fn handle_connection(mut stream: TcpStream) {
        loop {
            let mut buf_reader = BufReader::new(&mut stream);
            let mut line = String::new();
            match buf_reader.read_line(&mut line) {
                Ok(0) => {
                    return;
                }
                Ok(_) => {
                    let response = Self::get_task_value(line);
                    if let Some(r) = response {
                        stream.write(&[r]).unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("Unable to get command due to: {}", e);
                    return;
                }
            }
        }
    }

    fn get_task_value(buf: String) -> Option<u8> {
        let try_parse = || -> Result<u8, Box<dyn std::error::Error>> {
            let numbers: Vec<&str> = buf.trim().split(':').collect();
            let task_type = numbers.first().unwrap().parse::<u8>()?;
            let seed = numbers.last().unwrap().parse::<u64>()?;

            let result = Task::execute(task_type, seed);
            Ok(result)
        };

        match try_parse() {
            Ok(r) => Some(r),
            Err(_) => None
        }
    }
}
