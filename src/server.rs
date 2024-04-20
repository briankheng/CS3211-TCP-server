use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc};
use std::thread;
use std_semaphore::Semaphore;

use crate::task::{Task, TaskType};

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

        let sem = Arc::new(Semaphore::new(40));

        for stream in listener.unwrap().incoming() {
            match stream {
                Ok(stream) => {
                    let sem_clone = sem.clone();
                    thread::spawn(move || {
                        Self::handle_connection(stream, sem_clone);
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

impl Server {
    fn handle_connection(mut stream: TcpStream, mut sem: Arc<Semaphore>) {
        loop {
            let mut buf_reader = BufReader::new(&mut stream);
            let mut line = String::new();
            match buf_reader.read_line(&mut line) {
                Ok(0) => {
                    return;
                }
                Ok(_) => {
                    let response = Self::get_task_value(line, &mut sem);
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

    fn get_task_value(buf: String, sem: &mut Arc<Semaphore>) -> Option<u8> {
        let try_parse = || -> Result<u8, Box<dyn std::error::Error>> {
            let numbers: Vec<&str> = buf.trim().split(':').collect();
            let task_type = numbers.first().unwrap().parse::<u8>()?;
            let seed = numbers.last().unwrap().parse::<u64>()?;
            
            if TaskType::from_u8(task_type).unwrap() == TaskType::CpuIntensiveTask {
                let _guard = sem.access();
                let result = Task::execute(task_type, seed);
                return Ok(result);
            }
            let result = Task::execute(task_type, seed);
            Ok(result)
        };

        match try_parse() {
            Ok(r) => Some(r),
            Err(_) => None
        }
    }
}
