use std::collections::HashMap;

use futures::{FutureExt, StreamExt};
use futures::stream::SplitStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

use crate::{Client, Game, Games, GameState};

pub async fn new_game(username: String, ws: WebSocket, games: Games) {
    println!("Creating game and establishing client connection...");
    let (mut client_ws_rcv, client_sender) = establish_websocket_connection(ws);

    let (client_id, new_client) = create_client(username, client_sender);

    let (game_id, new_game) = create_game(client_id.clone(), new_client);

    if let Ok(mut editable_games) = games.try_lock() {
        editable_games.insert(game_id.clone(), new_game);
    } else {
        println!("Failed to get lock on games.");
    }

    println!("Game created {}", game_id);
    // TODO Send game id to user

    handle_messages(&mut client_ws_rcv, &client_id, &games, &game_id).await;

    remove_client(&games, &game_id, &client_id);
}

pub async fn join_game(username: String, ws: WebSocket, games: Games, game_id: String) {
    println!("Finding game and establishing client connection...");
    let (mut client_ws_rcv, client_sender) = establish_websocket_connection(ws);

    let (client_id, new_client) = create_client(username, client_sender);

    println!("FIND GAME");
    add_client_to_game(client_id.clone(), new_client, &games, &game_id);

    handle_messages(&mut client_ws_rcv, &client_id, &games, &game_id).await;

    remove_client(&games, &game_id, &client_id);
}

fn establish_websocket_connection(ws: WebSocket) -> (SplitStream<WebSocket>, UnboundedSender<Result<Message, warp::Error>>) {
    let (client_ws_sender, client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    return (client_ws_rcv, client_sender);
}

fn create_client(username: String, client_sender: UnboundedSender<Result<Message, warp::Error>>) -> (String, Client) {
    let client_id = Uuid::new_v4().to_simple().to_string();
    let new_client = Client {
        client_id: client_id.clone(),
        username,
        sender: Some(client_sender),
    };

    return (client_id, new_client);
}

fn create_game(client_id: String, client: Client) -> (String, Game) {
    let mut clients: HashMap<String, Client> = HashMap::new();
    clients.insert(client_id, client);

    let game_id = Uuid::new_v4().to_simple().to_string();
    let game_state = GameState { word_to_guess: None };
    let new_game = Game {
        game_id: game_id.clone(),
        game_state,
        clients,
    };

    return (game_id, new_game);
}

fn add_client_to_game(client_id: String, client: Client, games: &Games, game_id: &str) {
    if let Ok(mut editable_games) = games.try_lock() {
        match editable_games.get_mut(game_id) {
            Some(game) => {
                println!("ADD CLIENT");
                let clients = &mut game.clients;
                clients.insert(client_id.clone(), client);
            }
            None => {
                println!("DIDN'T FIND GAME");
                return // TODO Oh, no! Game not found! Return error?
            }
        }
    } else {
        println!("Failed to get lock on games.");
    }
}

async fn handle_messages(client_ws_rcv: &mut SplitStream<WebSocket>, client_id: &str, games: &Games, game_id: &str) {
    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error receiving message for id {}): {}", &client_id, e);
                break;
            }
        };
        handle_message(&game_id, &client_id, msg, &games).await;
    }
}

async fn handle_message(game_id: &str, client_id: &str, msg: Message, games: &Games) {
    println!("received message from {}: {:?}", client_id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    let editable_games = games.lock().await;
    println!("Finding all connected to the game");
    let _ =
        match editable_games.get(game_id) {
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
            }
            None => {
                println!("Game not found!");
                return
            }
        };
    return;
}

fn remove_client(games: &Games, game_id: &str, client_id: &str) {
    println!("Removing client '{}' from game", client_id);
    if let Ok(mut editable_games) = games.try_lock() {
        match editable_games.get_mut(game_id) {
            Some(game) => {
                let clients = &mut game.clients;
                clients.remove(client_id);
                println!("{} disconnected", client_id);
                // TODO Remove game when last client disconnects?
            }
            None => return // TODO Oh, no! Game not found! Return error?
        }
    }
}
