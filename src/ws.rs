use std::collections::HashMap;
use crate::{Client, Clients, Game, Games, GameState};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

pub async fn new_game(username: String, ws: WebSocket, games: Games) {
    println!("Creating game and establishing client connection... {:?}", ws);
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    let client_id = Uuid::new_v4().to_simple().to_string();
    let new_client = Client {
        client_id: client_id.clone(),
        username: username.clone(),
        sender: Some(client_sender),
    };

    let mut clients: HashMap<String, Client> = HashMap::new();
    clients.insert(client_id, new_client);

    let game_id = Uuid::new_v4().to_simple().to_string();
    let game_state = GameState { word_to_guess: None };
    let new_game = Game {
        game_id: game_id.clone(),
        game_state,
        clients,
    };

    games.lock().await.insert(game_id.clone(), new_game);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error receiving message for id {}): {}", client_id.clone(), e);
                break;
            }
        };
        client_msg(&game_id, &client_id, msg, &games).await;
    }
    clients.lock().await.remove(&client_id);
    // TODO Remove game when last client disconnects
    println!("{} disconnected", client_id);
}

pub async fn join_game(game_id: String, username: String, ws: WebSocket, games: Games) {
    println!("Finding game and establishing client connection... {:?}", ws);
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    let client_id = Uuid::new_v4().to_simple().to_string();
    let new_client = Client {
        client_id: client_id.clone(),
        username: username.clone(),
        sender: Some(client_sender),
    };

    // TODO find game
    let locked_games = games.lock().await;
    match locked_games.get(game_id) {
        Some(game) => {
            // TODO add user
            // TODO cannot borrow immutable clients
            game.clients.insert(client_id, new_client);
        }
        None => return // Oh, no! Return error?
    }

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error receiving message for id {}): {}", client_id.clone(), e);
                break;
            }
        };
        client_msg(&game_id, &client_id, msg, &games).await;
    }
    clients.lock().await.remove(&client_id);
    // TODO Remove game when last client disconnects
    println!("{} disconnected", client_id);
}

async fn client_msg(game_id: &str, client_id: &str, msg: Message, games: &Games) {
    println!("received message from {}: {:?}", client_id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };
    let locked = games.lock().await;
    match locked.get(game_id) {
        Some(game) => {
            match game.clients.get(client_id) {
                Some(v) => {
                    if let Some(sender) = &v.sender {
                        if message == "ping" || message == "ping\n" {
                            println!("sending pong");
                            let _ = sender.send(Ok(Message::text("pong")));
                        }
                    }
                }
                None => return
            }
        }
        None => return
    }

    let _ =
        match locked.get(game_id) {
            Some(game) => {
                game.clients
                    .filter(|client| && client.client_id != client_id)
                    .for_each(|client| if let Some(sender) = &client.sender {
                        println!("{} sending '{}' to {}", client_id, message, &client.client_id);
                        // TODO cannot borrow immutable sender
                        let _ = sender.send(Ok(Message::text(format!("{} from {}", message, &client.client_id))));
                    });
            }
            None => return
        };
    return;
}
