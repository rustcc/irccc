#![allow(unused_mut)]
#![allow(unused_must_use)]
extern crate websocket;
//extern crate openssl;

use std::comm;
use std::thread::Thread;
use std::io::{Listener, Acceptor};
use websocket::{WebSocketServer, WebSocketMessage};
//use websocket::client::WebSocketClient;
use websocket::header::WebSocketProtocol;
//use openssl::ssl::{SslContext, SslMethod};
//use openssl::x509::X509FileType;

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
            ctx.send(client_ref);
            let task_tx = tx.clone();

            Thread::spawn(move || {

                let mut client_captured = client.clone();

                for message in client.incoming_messages() {
                    // main body of websocket communication
                    // message is the msg object from client
                    // and here can do logic process after receiving msg
                    match message {
                        Ok(message) => {
                            println!("Recv [{}]: {}", id, message);

                            match message {
                                // Handle Ping messages by sending Pong messages
                                WebSocketMessage::Ping(data) => {
                                    let message = WebSocketMessage::Pong(data);
                                    let _ = client_captured.send_message(message);
                                    println!("Closed connection {}", id);
                                    // Close the connection
                                    break;
                                }
                                // Handle when the client wants to disconnect
                                WebSocketMessage::Close(_) => {
                                    // Send a close message
                                    let message = WebSocketMessage::Close(None);
                                    let _ = client_captured.send_message(message);
                                    println!("Closed connection {}", id);
                                    // Close the connection
                                    break;
                                }
                                _ => { }
                            }

                            task_tx.send("123");
                            //let message = WebSocketMessage::Text("Response from the server".to_string());
                            //let _ = client_captured.send_message(message);
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
        let msg = rx.recv();
        let client = crx.try_recv();
        match client {
            Ok(client) => all_clients.push(client),
            _ => {}
        }
        for client in all_clients.iter_mut() {
            let message = WebSocketMessage::Text("haha, im".to_string());
            //let mut mclient = client.clone();
            client.send_message(message);

        }
    }
}

