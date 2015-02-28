#![feature(phase)]
#[phase(plugin, link)] extern crate log;
extern crate websocket;

use std::comm;
use std::thread::Thread;
use std::io::{Listener, Acceptor};
use std::error;
use std::iter::count;

use websocket::{WebSocketServer, WebSocketMessage};
use websocket::header::WebSocketProtocol;


fn main() {
    let addr = "192.168.137.42:8080";
    match serve(addr) {
        Ok(_) => {println!("server exited")},
        Err(msg) => {println!("server panic with: {}", msg.description())}
    }
}

fn serve(addr: &'static str) -> Result<(), Box<error::Error>> {
    
    
    let server = try!(WebSocketServer::bind(addr));
    let mut acceptor = try!(server.listen());

    let mut all_clients = Vec::new();

    let (tx, rx) = comm::channel();
    let (ctx, crx) = comm::channel();
    
    Thread::spawn(move || {

        for (req, id) in acceptor.incoming().zip(count(1u,1u)) {
            debug!("Connection [{}]", id);
            
            let request;
            match req {
                Ok(req) => request = req,
                Err(msg) => {
                    warn!("unwrap request fail! {}", msg);
                    continue;
                }
            }

            // Let's also check the protocol - if it's not what we want, then fail the connection
            if request.protocol().is_none() || !request.protocol().unwrap().as_slice().contains(&"irccc".to_string()) {
                warn!("got bad request with id {}", id);
                let response = request.fail();
                let _ = response.send_into_inner();
                continue;
            }

            let mut response = request.accept(); // Generate a response object
            response.headers.set(WebSocketProtocol(vec!["irccc".to_string()])); // Send a Sec-WebSocket-Protocol header
            let mut client; // create a client
            match response.send() {
                Ok(c) => client = c,
                Err(msg) => {
                    warn!("cannot create client for id: {}, err: {}", id, msg);
                    continue;
                }
            }

            ctx.send(client.clone());
            
            // send the welcome msg
            let message = WebSocketMessage::Text("Welcome to rust irccc server".to_string());
            let _ = client.send_message(message);

            let task_tx = tx.clone();
            Thread::spawn(move || {

                while let Some(Ok(message)) = client.incoming_messages().next() {
                    // main body of websocket communication
                    // message is the msg object from client
                    // and here can do logic process after receiving msg
                    info!("Recv [{}]: {}", id, message);
                    match message {
                        // Handle Ping messages by sending Pong messages
                        WebSocketMessage::Ping(data) => {
                            let message = WebSocketMessage::Pong(data);
                            let _ = client.send_message(message);
                            println!("Closed connection {}", id);
                            // Close the connection
                            break;
                        }
                        // Handle when the client wants to disconnect
                        WebSocketMessage::Close(_) => {
                            // Send a close message
                            let message = WebSocketMessage::Close(None);
                            let _ = client.send_message(message);
                            println!("Closed connection {}", id);
                            // Close the connection
                            break;
                        }
                        _ => { }
                    }
                    task_tx.send("123");
                }
            }).detach();
        }
    }).detach();

    loop {
        select!(
            msg = rx.recv() => {
                println!("--- {}", msg);
            },
            client = crx.recv() => {
                all_clients.push(client);
            });
        println!("ourter_all_clients.len {}", all_clients.len());
    }
}

