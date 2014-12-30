#![allow(unused_mut)]
#![allow(unused_must_use)]
extern crate websocket;
//extern crate openssl;

use std::comm;
use std::thread::Thread;
use std::io::{Listener, Acceptor};
use websocket::{WebSocketServer, WebSocketMessage};
use websocket::client::WebSocketClient;
use websocket::header::WebSocketProtocol;
//use openssl::ssl::{SslContext, SslMethod};
//use openssl::x509::X509FileType;

struct ConnDirective<S, R, C, E, W, M> {
    cmd: &'static str,
    client: WebSocketClient<S, R, C, E, W, M>
}



fn main() {
    /*
       let mut context = SslContext::new(SslMethod::Tlsv1).unwrap();
       let _ = context.set_certificate_file(&(Path::new("cert.pem")), X509FileType::PEM);
       let _ = context.set_private_key_file(&(Path::new("key.pem")), X509FileType::PEM);

       let server = WebSocketServer::bind_secure("127.0.0.1:2794", &context).unwrap();
       */
    let server = WebSocketServer::bind("127.0.0.1:8080").unwrap();

    let mut acceptor = server.listen().unwrap();

    let mut all_clients = Vec::new();
    let (tx, rx) = comm::channel();
    let (ctx, crx) = comm::channel();
    let mut id = 0u;

    Thread::spawn(move || {
        for request in acceptor.incoming() {
            id += 1;
            println!("Connection [{}]", id);
            let request = request.unwrap();

            // Let's also check the protocol - if it's not what we want, then fail the connection
            if request.protocol().is_none() || !request.protocol().unwrap().as_slice().contains(&"irccc".to_string()) {
                println!("bad request");
                let response = request.fail();
                response.send_into_inner();
                return;
            }

            let mut response = request.accept(); // Generate a response object
            response.headers.set(WebSocketProtocol(vec!["irccc".to_string()])); // Send a Sec-WebSocket-Protocol header
            let mut client = response.send().unwrap(); // create a client
            // send the welcome msg
            let message = WebSocketMessage::Text("Welcome to rust irccc server".to_string());
            client.send_message(message);

            let mut client_ref = client.clone();
            let mut connection_tx = ctx.clone();
            let conndir_obj = ConnDirective{ cmd: "add", client: client_ref };
            connection_tx.send(conndir_obj);
            
            let task_tx = tx.clone();

            Thread::spawn(move || {

                let mut client_captured = client.clone();

                for message in client.incoming_messages() {
                    // main body of websocket communication
                    // message is the msg object from client
                    // and here can do logic process after receiving msg
                    match message {
                        Ok(message) => {

                            match message {
                                // Handle Ping messages by sending Pong messages
                                WebSocketMessage::Ping(data) => {
                                    let message = WebSocketMessage::Pong(data);
                                    let _ = client_captured.send_message(message);
                                    println!("Closed connection {}", id);
                                    // Close the connection
                                    break;
                                },
                                // Handle when the client wants to disconnect
                                WebSocketMessage::Close(_) => {
                                    // Send a close message
                                    let message = WebSocketMessage::Close(None);
                                    let _ = client_captured.send_message(message);
                                    println!("Closed connection {}", id);
                                    // here, send close directive to connection collector
                                    let conndir_obj = ConnDirective{ cmd: "remove", client: client_captured };
                                    
                                    connection_tx.send(conndir_obj);
                            
                                    // Close the connection
                                    break;
                                },
                                _ => { 
                                    // main logic
                                    task_tx.send(message);
                                
                                }
                            }
                        }
                        Err(err) => {
                            println!("Error [{}]: {}",id, err);
                            break;
                        }
                    }

                }
            }).detach();
        }
    }).detach();

    loop {
        select! (
            clientdir = crx.recv() => {
                // TODO: here should contain add, remove
                // add new client to connection collector
                match clientdir.cmd {
                    "add" => {
                        all_clients.push(clientdir.client);
                    },
                    "remove" => {
                        // remove from dir
                    }

                }
            },
            msg = rx.recv() => { 
                // send to all clients
                for client in all_clients.iter_mut() {
                    //let message = WebSocketMessage::Text("haha, im".to_string());
                    client.send_message(msg.clone());
                }
            }
        )
            
    }
}

