use std::collections::HashMap;
use crate::{Client, Game, Games, GameState};
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
    clients.insert(client_id.clone(), new_client);

    let game_id = Uuid::new_v4().to_simple().to_string();
    let game_state = GameState { word_to_guess: None };
    let new_game = Game {
        game_id: game_id.clone(),
        game_state,
        clients,
    };

    games.lock().await.insert(game_id.clone(), new_game);
    println!("Game created {}", game_id);
    // TODO Send game id to user

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

    let mut locked_games = games.lock().await;
    match locked_games.get_mut(&game_id) {
        Some(game) => {
            let clients = &mut game.clients;
            clients.remove(&client_id);
            println!("{} disconnected", client_id);
            // TODO Remove game when last client disconnects
        }
        None => return // TODO Oh, no! Game not found! Return error?
    }
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
    println!("FIND GAME");
    let mut locked_games = games.lock().await;
    match locked_games.get_mut(&game_id) {
        Some(game) => {
            println!("ADD CLIENT");
            let clients = &mut game.clients;
            clients.insert(client_id.clone(), new_client);
        }
        None => {
            println!("DIDN'T FIND GAME");
            return // TODO Oh, no! Game not found! Return error?
        }
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

    match locked_games.get_mut(&game_id) {
        Some(game) => {
            let clients = &mut game.clients;
            clients.remove(&client_id);
            println!("{} disconnected", client_id);
            // TODO Remove game when last client disconnects
        }
        None => return // TODO Oh, no! Game not found! Return error?
    }
}

async fn client_msg(game_id: &str, client_id: &str, msg: Message, games: &Games) {
    println!("received message from {}: {:?}", client_id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    // println!("Ping pong?");
    // TODO this will wait forever
    let locked = games.lock().await;
    // match locked.get(game_id) {
    //     Some(game) => {
    //         println!("Game found. Finding clients.");
    //         match game.clients.get(client_id) {
    //             Some(client) => {
    //                 if let Some(sender) = &client.sender {
    //                     if message == "ping" || message == "ping\n" {
    //                         println!("sending pong");
    //                         let _ = sender.send(Ok(Message::text("pong")));
    //                     }
    //                 }
    //             }
    //             None => {
    //                 println!("No clients found!");
    //                 return
    //             }
    //         }
    //     }
    //     None => {
    //         println!("No matching game found!");
    //         return
    //     }
    // }

    println!("Finding all connected to the game");
    let _ =
        match locked.get(game_id) {
            Some(game) => {
                println!("Game found.");
                for (current_client_id, client) in &game.clients {
                    println!("Iterating client {}, for client {} message.", current_client_id, client.client_id);
                    if current_client_id != client_id {
                        match &client.sender {
                            Some(sender) => {
                                println!("{} sending '{}' to {}", client_id, message, &client.client_id);
                                let _ = sender.send(Ok(Message::text(format!("{} from {}", message, &client.client_id))));
                            }
                            None => return
                        }
                    } else {
                        println!("Same client. Not sending message.")
                    }
                }
                // game.clients
                //     .filter(|client| && client.client_id != client_id)
                //     .for_each(|client| if let Some(mut sender) = &client.sender {
                //         println!("{} sending '{}' to {}", client_id, message, &client.client_id);
                //         let _ = sender.send(Ok(Message::text(format!("{} from {}", message, &client.client_id))));
                //     });
            }
            None => {
                println!("Game not found!");
                return
            }
        };
    return;
}
