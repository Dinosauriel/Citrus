use std::io;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread;

// fetch all the lines from the stdin channel and return them
pub fn commands(receiver: &Receiver<String>) -> Vec<String> {
    let mut commands = Vec::<String>::new();
    loop {
        match receiver.try_recv() {
            Ok(key) => commands.push(String::from(key)),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => panic!("stdin channel disconnected"),
        }
    }
    commands
}

// start a new thread that waits for input
pub fn stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || 
        loop {
            let mut buffer = String::new();
            // blocking readline call
            io::stdin().read_line(&mut buffer).unwrap();
            tx.send(buffer).unwrap();
        });
    rx
}

