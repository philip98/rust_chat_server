use std::collections::HashMap;
use std::io::{BufReader, BufRead, BufWriter, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

enum Message {
    ClientMessage(SocketAddr, String),
    ServerMessage(String)
}

type ClientList = HashMap<SocketAddr, BufWriter<TcpStream>>;

fn accept_th(tx: Sender<Message>, clients: Arc<Mutex<ClientList>>) {
    let listener = TcpListener::bind("127.0.0.1:65521").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(socket) => {
                let addr = socket.peer_addr().unwrap();
                let s2 = socket.try_clone().unwrap();
                let tx2 = tx.clone();
                let cl2 = clients.clone();
                {
                    let mut cl = (*clients).lock().unwrap();
                    (*cl).insert(addr, BufWriter::new(socket));
                }
                println!("New connection: {}", addr);
                tx.send(Message::ServerMessage(format!("--- {} joined ---", addr))).unwrap();
                thread::spawn(move || rx_th(s2, tx2, cl2));
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    std::process::exit(0);
}

fn rx_th(socket: TcpStream, tx: Sender<Message>, clients: Arc<Mutex<ClientList>>) {
    let addr = socket.peer_addr().unwrap();
    let rd = BufReader::new(socket);
    for line in rd.lines() {
        match line {
            Ok(line) => {
                {
                    println!("{}: {}", addr, &line);
                }
                tx.send(Message::ClientMessage(addr, line)).unwrap();
            },
            Err(e) => {
                println!("Read error: {}", e);
            }
        }
    }
    println!("{} is leaving", addr);
    tx.send(Message::ServerMessage(format!("--- {} is leaving ---", addr))).unwrap();
    let mut cl = (*clients).lock().unwrap();
    (*cl).remove(&addr);
}

fn tx_th(rx: Receiver<Message>, clients: Arc<Mutex<ClientList>>) {
    for msg in rx {
        match msg {
            Message::ClientMessage(addr, msg) => {
                let mut cl = (*clients).lock().unwrap();
                for (&cl_addr, wr) in cl.iter_mut() {
                    if cl_addr != addr {
                        write!(wr, "{}: {}\n", addr, &msg).unwrap();
                        wr.flush().unwrap();
                    }
                }
            },
            Message::ServerMessage(msg) => {
                let mut cl = (*clients).lock().unwrap();
                for (_, wr) in cl.iter_mut() {
                    write!(wr, "{}\n", &msg).unwrap();
                    wr.flush().unwrap();
                }
            }
        }
    }
    std::process::exit(0);
}

fn main() {
    let (tx, rx) = channel::<Message>();
    let clients = Arc::new(Mutex::new(HashMap::new()));
    let clients2 = clients.clone();
    thread::spawn(move || tx_th(rx, clients2));
    accept_th(tx, clients);
}
