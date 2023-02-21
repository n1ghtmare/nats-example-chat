use std::env;

use async_nats::Client;
use futures::StreamExt;
use tokio::{io::AsyncBufReadExt, task::JoinHandle};

// TODO: Add a proper help message, should do just fine for the demo
const HELP_MESSAGE: &str = "We need a help message here...";

async fn handle_send(client: Client, room_name: String, nickname: String) -> JoinHandle<()> {
    // We send/publish messages here
    tokio::spawn(async move {
        loop {
            let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
            let mut buffer = String::new();

            let _ = reader.read_line(&mut buffer).await;

            let contents = buffer.trim();
            let message = format!("{nickname}: {contents}");

            let _ = client.publish(room_name.to_string(), message.into()).await;
            buffer.clear();
        }
    })
}

async fn handle_receive(client: Client, room_name: String, nickname: String) -> JoinHandle<()> {
    let mut subscriber = client.subscribe(room_name.to_string()).await.unwrap();

    tokio::spawn(async move {
        while let Some(message) = subscriber.next().await {
            let message = std::str::from_utf8(&message.payload).unwrap();

            // TODO: There has to be a better way to identify messages that don't originate from the current user
            if !message.contains(&nickname) {
                println!("> {message}");
            }
        }
    })
}

#[tokio::main]
async fn main() -> Result<(), async_nats::Error> {
    // nats-example-chat "chat_room_name" "nickname"
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || &args[1] == "--help" || &args[1] == "-?" {
        println!("{HELP_MESSAGE}");
        return Ok(());
    }

    // TODO: Handle invalid arguments?

    // TODO: Is this the best way to pass data to threads? Is this correct?
    // TODO: Lots and lots of cloning, I gotta look into this
    let room_name = args[1].clone();
    let nickname = args[2].clone();

    let client = async_nats::connect("0.0.0.0:4222").await?;

    let send_handle = handle_send(client.clone(), room_name.clone(), nickname.clone()).await;
    let receive_handle = handle_receive(client, room_name, nickname);

    let _ = tokio::join!(send_handle, receive_handle);

    Ok(())
}
