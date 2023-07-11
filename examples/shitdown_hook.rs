use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::sleep;
use std::time::Duration;
use ctor::dtor;

#[dtor]
fn tear_down() {
    println!("tear_down.enter");

    let guard = stream.lock();
    let mut incoming = guard.unwrap();
    let incoming = incoming.as_mut();
    let incoming = incoming.unwrap();
    println!("tear_down: writing goodbye");
    incoming.write_all("goodbye!".as_bytes());
    println!("tear_down: flushing");
    incoming.flush();
    println!("tear_down.exit");
}

enum Message {
    Text(String),
    End,
}

enum AppStatus {
    InProgress,
    TearDownRequested,
    TearDownCompleted,
}

// static outgoing: RwLock<VecDeque<Message>> = RwLock::new(VecDeque::new());
static status: Mutex<AppStatus> = Mutex::new(AppStatus::InProgress);
static stream: Mutex<Option<TcpStream>> = Mutex::new(None);

fn main() {
    println!("main.enter");
    let listener = TcpListener::bind("localhost:8080").unwrap();
    for mut incoming in listener.incoming().flatten() {
        println!("main: connection accepted");
        println!("main: writing hello");
        incoming.write_all("hello!".as_bytes());
        println!("main: flushing");
        incoming.flush();
        *stream.lock().unwrap() = Some(incoming.try_clone().unwrap());
        loop {
            println!("main: waits");
            // if let AppStatus::TearDownRequested = *status.lock().unwrap() {
                // break
            // }
            sleep(Duration::from_secs_f32(1.0));
        }
        println!("main: writing goodbye");
        incoming.write_all("goodbye!".as_bytes());
        println!("main: flushing");
        incoming.flush();
        *status.lock().unwrap() = AppStatus::TearDownCompleted;
        println!("main: connection dropped");
    }
    println!("main.exit");
}