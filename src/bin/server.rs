use tokio::{
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}, sync::broadcast
};

use serde::{Serialize, Deserialize};

use chrono::Local;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// this is an attribute instructing the compiler to auto-generate impls for the 4 traits

struct ChatMessage{
    username: String,
    content: String,
    timestamp: String,
    message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageType{
    Usermessage,
    SystemNotification, 
}

// What is tokio?
// Async runtime for the rust language
// Async programming allows for the program to do other things while waiting for a task
// to finish
#[tokio::main]


async fn main() -> Result<(),Box<dyn Error>>{
    let listener = TcpListener::bind("127.0.0.1:8082").
    await?;

    println!("Terminal Chat App");
    println!("Host: 127.0.0.1");
    println!("Port: 8082");

    // Create a broadcast channel for message distribution

    let (tx, _) = broadcast::channel::<String>(100);

    loop {
        let (socket, addr) = listener.accept().await?;

        // Display the connection info
        println!("|-[{}] New connection", Local::now().format("%H;%M;%S"));
        println!("^-Address: {}", addr);

        // Clone sender for this connection and sunscribe a receiver
        let tx  = tx.clone();
        let rx  = tx.subscribe();

        tokio::spawn(async move{
            handle_connection(socket, tx, rx).await
        });
    }
    }

    async fn handle_connection(
        mut socket: TcpStream,
        tx: broadcast::Sender<String>,
        mut rx: broadcast::Receiver<String>,
    ){
    // Splitting the socket into reading and writer
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut username: String = String::new();

    // Read the username sent by the client
    reader.read_line(&mut username).await.unwrap();
    let username = username.trim().to_string();

    // Send a sys notification indicating the user has joined the chat
    let join_msg = ChatMessage{
        username: username.clone(),
        content: "joined the chat".to_string(),
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        message_type: MessageType::SystemNotification,
    };
    let join_json = serde_json::to_string(&join_msg).unwrap();
    tx.send(join_json).unwrap();

    let mut line = String::new();
    loop{
        tokio::select! {
            result = reader.read_line(&mut line) =>{
                if result.unwrap() == 0 {
                    break;
                }
                // Create and broadcast user message
                let msg = ChatMessage{
                    username: username.clone(),
                    content: line.trim().to_string(),
                    timestamp: Local::now().format("%H:%M:%S").to_string(),
                    message_type: MessageType::Usermessage
                };
                let json = serde_json::to_string(&msg).unwrap();
                tx.send(json).unwrap();
                line.clear();
            }
            // Handle the incoming broadcasts and send them to the client
            result = rx.recv() => {
                let msg = result.unwrap();
                writer.write_all(msg.as_bytes()).await.unwrap();
                writer.write_all(b"\n").await.unwrap();

            }
        }
    }
    let leave_msg = ChatMessage{
        username: username.clone(),
        content: "left the chat".to_string(),
        timestamp: Local::now().format("%H;%M;%S").to_string(),
        message_type: MessageType::SystemNotification,
    };
    let leave_json = serde_json::to_string(&leave_msg).unwrap();
    tx.send(leave_json).unwrap();

    // Log disconnect to console
    println!("-[{}] {} disconnected", Local::now().format("%H:%M:%S").to_string(), username);
    
}