use std::collections::HashMap;

use futures::{FutureExt, StreamExt};
use futures::stream::SplitStream;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

use crate::{Client, Game, Games, GameState};

use crate::words;

#[derive(Debug, Serialize, Deserialize)]
struct ActionMessage {
    action: String,
}

pub async fn new_game(username: String, ws: WebSocket, games: Games) {
    println!("Creating game and establishing client connection...");
    let (mut client_ws_rcv, client_sender) = establish_websocket_connection(ws);

    let new_game_id = create_new_game_id(&games);

    let (client_id, new_client) = create_client(username, client_sender);

    let new_game = create_game_with_id(&new_game_id, client_id.clone(), new_client.clone());

    if let Ok(mut editable_games) = games.try_lock() {
        editable_games.live_games.insert(new_game_id.clone(), new_game);
    } else {
        println!("Failed to get lock on games.");
    }

    println!("Game created {}", &new_game_id);
    let new_game_message = json!({
                    "event": "new_game",
                    "payload": {"id": new_game_id}
                });
    send_message(&new_client, &*new_game_message.to_string()).await;

    handle_messages(&mut client_ws_rcv, &client_id, &games, &new_game_id).await;

    remove_client(&games, &new_game_id, &client_id);
}

pub async fn join_game(username: String, ws: WebSocket, games: Games, game_id: String) {
    println!("Finding game and establishing client connection...");
    let (mut client_ws_rcv, client_sender) = establish_websocket_connection(ws);

    let (client_id, new_client) = create_client(username, client_sender);

    println!("FIND GAME");
    add_client_to_game(client_id.clone(), new_client, &games, &game_id).await;

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

fn create_new_game_id(games: &Games) -> String {
    return if let Ok(mut editable_games) = games.try_lock() {
        editable_games.games_created += 1;
        let game_id = (1000 + editable_games.games_created).to_string();
        game_id
    } else {
        // TODO Errors and error handling
        Uuid::new_v4().to_simple().to_string()
    };
}

fn create_client(username: String, client_sender: UnboundedSender<Result<Message, warp::Error>>) -> (String, Client) {
    // TODO Until I learn to mock ID generator for unit tests, use simple predictable user IDs.
    // let client_id = Uuid::new_v4().to_simple().to_string();
    let client_id = format!("{}_id", username);
    let new_client = Client {
        client_id: client_id.clone(),
        username,
        sender: Some(client_sender),
    };

    return (client_id, new_client);
}

fn create_game_with_id(game_id: &str, client_id: String, client: Client) -> Game {
    let mut clients: HashMap<String, Client> = HashMap::new();
    clients.insert(client_id, client.clone());

    let game_state = GameState { word_to_guess: None, client_turns: vec!(client) };
    let new_game = Game {
        game_id: game_id.to_string(),
        game_state,
        clients,
    };

    return new_game;
}

async fn add_client_to_game(client_id: String, client: Client, games: &Games, game_id: &str) {
    if let Ok(mut editable_games) = games.try_lock() {
        match editable_games.live_games.get_mut(game_id) {
            Some(game) => {
                println!("ADD CLIENT");

                // TODO Typed events?
                let join_message = json!({
                    "event": "join",
                    "payload": {
                        "id": client.client_id,
                        "name": client.username
                    }
                });
                for (_, client_to_notify) in &game.clients {
                    send_message(client_to_notify, &*join_message.to_string()).await;
                }

                let clients = &mut game.clients;
                clients.insert(client_id.clone(), client.clone());

                let game_state = &mut game.game_state;
                game_state.client_turns.push(client);
            }
            None => {
                println!("DIDN'T FIND GAME");
                return; // TODO Oh, no! Game not found! Return error?
            }
        }
    } else {
        println!("Failed to get lock on games.");
    }
}

async fn send_message(client: &Client, message: &str) {
    match &client.sender {
        Some(sender) => {
            println!("sending '{}' to {} ({})", message, client.username, client.client_id);
            let _ = sender.send(Ok(Message::text(format!("{}", message))));
        }
        None => return
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

async fn start_next_round(game_id: &str, games: &Games) {
    let word = if let Ok(readable_games) = games.try_lock() {
        let word = match &readable_games.test_word {
            Some(w) => w.clone(),
            None => words::get_random_word(),
        };

        println!("Word! {}", word.clone());
        word
    } else {
        String::from("Could not get a word")
    };

    if let Ok(mut editable_games) = games.try_lock() {
        match editable_games.live_games.get_mut(game_id) {
            Some(game) => {
                let game_state = &mut game.game_state;
                game_state.word_to_guess = Some(word.clone());

                let guesser = game_state.client_turns.remove(0);
                let you_are_guesser_message = json!({
                    "event": "new_round",
                    "payload": {"role": "guesser"}
                });
                send_message(&guesser, &*you_are_guesser_message.to_string()).await;
                let hinters = game_state.client_turns.clone();
                let you_are_hinter_message = json!({
                    "event": "new_round",
                    "payload": {
                        "role": "hinter",
                        "word": word
                    }
                });
                for hinter in hinters {
                    send_message(&hinter, &*you_are_hinter_message.to_string()).await;
                }

                game_state.client_turns.push(guesser);
            }
            None => return // TODO Oh, no! Game not found! Return error?
        }
    }
}

async fn handle_message(game_id: &str, client_id: &str, msg: Message, games: &Games) {
    println!("received message from {}: {:?}", client_id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    // parse if possible
    match from_str::<ActionMessage>(message) {
        Ok(action_message) => {
            println!("Parsed action: {:?}", action_message);

            match &*action_message.action {
                "start_next_round" => start_next_round(game_id, games).await,
                unknown_action => println!("Unknown action: {}", unknown_action)
            }
        }
        Err(e) => {
            println!("Couldn't parse '{:?}' as Action", e);

            let editable_games = games.lock().await;
            println!("Finding all connected to the game");
            let _ =
                match editable_games.live_games.get(game_id) {
                    Some(game) => {
                        println!("Game found.");
                        for (current_client_id, client) in &game.clients {
                            println!("Iterating client {}, for client {} message.", current_client_id, client.client_id);
                            if current_client_id != client_id {
                                match &client.sender {
                                    Some(sender) => {
                                        println!("{} sending '{}' to {}", client_id, message, &client.client_id);
                                        let _ = sender.send(Ok(Message::text(format!("{}", message))));
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
                        return;
                    }
                };
        }
    }

    return;
}

fn remove_client(games: &Games, game_id: &str, client_id: &str) {
    println!("Removing client '{}' from game", client_id);
    if let Ok(mut editable_games) = games.try_lock() {
        match editable_games.live_games.get_mut(game_id) {
            Some(game) => {
                let clients = &mut game.clients;
                clients.remove(client_id);

                let game_state = &mut game.game_state;
                game_state.client_turns.retain(|c| c.client_id != client_id);
                println!("{} disconnected", client_id);
                // TODO Remove game when last client disconnects?
            }
            None => return // TODO Oh, no! Game not found! Return error?
        }
    }
}
