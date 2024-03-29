use std::collections::HashMap;

use futures::{FutureExt, StreamExt};
use futures::stream::SplitStream;
use itertools::Itertools;
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
pub struct ActionMessage {
    pub action: Action,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Action {
    SkipWordAction(SkipWord),
    StartNextRoundAction(StartNextRound),
    HintAction(Hint),
    GuessAction(Guess),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SkipWord {
    pub skip_word: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StartNextRound {
    pub start_next_round: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Hint {
    pub hint: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Guess {
    pub guess: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct ClientAndHint {
    client: String,
    hint: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ClientIdAndName {
    pub id: String,
    pub username: String,
}

pub async fn new_game(username: String, ws: WebSocket, games: Games) {
    println!("Creating game and establishing client connection...");
    let (mut client_ws_rcv, client_sender) = establish_websocket_connection(ws);

    let new_game_id = create_new_game_id(&games);

    let (client_id, new_client) = create_client(username.clone(), client_sender);

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

    send_message(&new_client, &*user_data_message(&client_id, &username)).await;

    handle_messages(&mut client_ws_rcv, &client_id, &games, &new_game_id).await;

    remove_client(&games, &new_game_id, &client_id).await;
}

fn user_data_message(client_id: &str, username: &str) -> String {
    return json!({
                    "event": "your_data",
                    "payload": {"id": client_id,
                                "username": username}
                }).to_string();
}

fn other_clients_message(clients: &Vec<Client>) -> String {
    let other_players = clients.clone().into_iter()
        .map(|client| ClientIdAndName {
            id: client.client_id,
            username: client.username,
        })
        .collect::<Vec<_>>();
    return json!({
            "event": "other_players",
            "payload": other_players
        }).to_string();
}

pub async fn join_game(username: String, ws: WebSocket, games: Games, game_id: String) {
    println!("Finding game and establishing client connection...");
    let (mut client_ws_rcv, client_sender) = establish_websocket_connection(ws);

    let (client_id, new_client) = create_client(username.clone(), client_sender);

    println!("FIND GAME");
    add_client_to_game(client_id.clone(), new_client.clone(), &games, &game_id).await;

    send_message(&new_client, &*user_data_message(&client_id, &username)).await;

    handle_messages(&mut client_ws_rcv, &client_id, &games, &game_id).await;

    remove_client(&games, &game_id, &client_id).await;
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

#[cfg(not(test))]
fn create_client_id(_username: String) -> String {
    return Uuid::new_v4().to_simple().to_string();
}

#[cfg(test)]
fn create_client_id(username: String) -> String {
    return format!("{}_id", username);
}

fn create_client(username: String, client_sender: UnboundedSender<Result<Message, warp::Error>>) -> (String, Client) {
    let client_id = create_client_id(username.clone());
    let new_client = Client {
        client_id: client_id.clone(),
        hint: None,
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
                        "username": client.username
                    }
                });
                // Notify others of a new player
                for (_, client_to_notify) in &game.clients {
                    send_message(client_to_notify, &*join_message.to_string()).await;
                }

                // Notify new player of others already joined
                if &game.clients.len() > &0usize {
                    send_message(&client, &*other_clients_message(&game.game_state.client_turns)).await;
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
    };

    return;
}

async fn send_message(client: &Client, message: &str) {
    match &client.sender {
        Some(sender) => {
            println!("sending '{}' to {} ({})", message, client.username, client.client_id);
            let _ = sender.send(Ok(Message::text(String::from(message))));
        }
        None => return
    };

    return;
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
    };

    return;
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
            println!("Parsed ActionMessage: {:?}", action_message);

            match action_message.action {
                Action::SkipWordAction(_) => start_next_round(game_id, games, false).await,
                Action::StartNextRoundAction(_) => start_next_round(game_id, games, true).await,
                Action::HintAction(hint) => add_hint(client_id, &hint.hint, game_id, games).await,
                Action::GuessAction(guess) => check_guess(guess.guess, game_id, games).await,
            }
        }
        Err(e) => {
            println!("Couldn't parse '{:?}' as ActionMessage.", e);
            println!("Games {:?}", games);
        }
    };

    return;
}

async fn start_next_round(game_id: &str, games: &Games, roll_roles: bool) {
    let word = if let Ok(current_games) = games.try_lock() {
        let word = match &current_games.test_word {
            Some(w) => w.clone(),
            None => words::get_random_word(),
        };

        println!("Word! {}", word.clone());
        word
    } else {
        String::from("Could not get a word.")
    };

    if let Ok(mut editable_games) = games.try_lock() {
        match editable_games.live_games.get_mut(game_id) {
            Some(game) => {
                let game_state = &mut game.game_state;
                game_state.word_to_guess = Some(word.clone());

                let guesser_index: usize = get_guesser_index(&game_state, roll_roles);
                let guesser = game_state.client_turns.remove(guesser_index);
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
                        "word": word,
                        "guesser": guesser.client_id
                    }
                });
                for hinter in hinters {
                    send_message(&hinter, &*you_are_hinter_message.to_string()).await;
                }

                game_state.client_turns.push(guesser.clone());

                // clear old hints
                let clients = &mut game.clients;
                for (_, client) in clients {
                    client.hint = None;
                }
            }
            None => return // TODO Oh, no! Game not found! Return error?
        }
    } else {
        println!("Could not get lock for game state.");
    };

    return;
}

fn get_guesser_index(game_state: &GameState, roll_roles: bool) -> usize {
    if roll_roles {
        0
    } else {
        game_state.client_turns.len() - 1
    }
}

async fn add_hint(client_id: &str, hint: &str, game_id: &str, games: &Games) {
    println!("{} {}", client_id, hint);

    if let Ok(mut editable_games) = games.try_lock() {
        match editable_games.live_games.get_mut(game_id) {
            Some(game) => {
                let clients = &mut game.clients;
                match clients.get_mut(client_id) {
                    Some(client) => client.hint = Some(String::from(hint)),
                    None => println!("Cloud not find client with id '{}' for storing hint.", client_id)
                };

                let hint_received_message = json!({
                        "event": "hint_received",
                        "payload": {"client": client_id}
                    });
                for (_, client) in clients {
                    if client.client_id != client_id {
                        send_message(client, &*hint_received_message.to_string()).await;
                    }
                }

                if is_all_hints_given(&game.clients) {
                    println!("All hints given!");

                    let (unique_hinter_clients, duplicate_hinter_clients, duplicate_hinter_ids) =
                        uniques_and_duplicates(game.clients.clone());

                    if let Some((guesser, hinters)) = game.game_state.client_turns.split_last() {
                        // To guesser
                        let hints_to_guesser_message = json!({
                            "event": "all_hints_to_guesser",
                            "payload": {"hints": unique_hinter_clients,
                            "usersWithDuplicates": duplicate_hinter_ids
                           }
                        });
                        send_message(guesser, &*hints_to_guesser_message.to_string()).await;

                        // To hinters
                        let hints_to_hinters_message = json!({
                            "event": "all_hints",
                            "payload": {"duplicates": duplicate_hinter_clients,
                                        "hints": unique_hinter_clients
                                       }
                        });
                        for hinter in hinters {
                            send_message(hinter, &*hints_to_hinters_message.to_string()).await
                        }
                    } else {
                        println!("Cloud not find guesser and hinters!")
                    }
                }
            }
            None => return // TODO Oh, no! Game not found! Return error?
        }
    } else {
        println!("Could not get lock for game state.");
    };

    return;
}

fn is_all_hints_given(clients: &HashMap<String, Client>) -> bool {
    return clients.iter().filter(|(_, client)| client.hint != None).count() == clients.len() - 1;
}

fn uniques_and_duplicates(clients: HashMap<String, Client>) -> (Vec<ClientAndHint>, Vec<ClientAndHint>, Vec<String>) {
    let grouped_by_hint = group_by_hint(clients);

    let unique_hinters: Vec<Client> = filter_unique_hinters(&grouped_by_hint);
    let unique_hinter_clients: Vec<ClientAndHint> = as_client_and_hints(unique_hinters);

    let duplicate_hinters: Vec<Client> = filter_duplicate_hinters(&grouped_by_hint);
    let duplicate_hinter_ids = duplicate_hinters.iter()
        .map(|client| client.client_id.clone())
        .sorted()
        .collect::<Vec<_>>();
    let duplicate_hinter_clients: Vec<ClientAndHint> = as_client_and_hints(duplicate_hinters);

    return (unique_hinter_clients, duplicate_hinter_clients, duplicate_hinter_ids);
}

fn group_by_hint(clients: HashMap<String, Client>) -> HashMap<Option<String>, Vec<Client>> {
    return clients
        .into_iter()
        .map(|(_, client)| client)
        .filter(|client| client.hint != None)
        .into_grouping_map_by(|client| Some(client.hint.clone().unwrap().to_lowercase()))
        .collect::<Vec<_>>();
}

fn filter_unique_hinters(grouped_by_hint: &HashMap<Option<String>, Vec<Client>>) -> Vec<Client> {
    return grouped_by_hint.into_iter()
        .fold(vec!(),
              |mut acc, (_, clients_with_same_hint)| {
                  if clients_with_same_hint.len() == 1 {
                      let client_with_unique_hint = clients_with_same_hint.get(0).unwrap();
                      acc.push(client_with_unique_hint.clone());
                      acc
                  } else {
                      acc
                  }
              },
        );
}

fn filter_duplicate_hinters(grouped_by_hint: &HashMap<Option<String>, Vec<Client>>) -> Vec<Client> {
    let init_acc: Vec<Client> = vec!();
    return grouped_by_hint.into_iter()
        .fold(init_acc,
              |mut acc, (_, clients_with_same_hint)| {
                  if clients_with_same_hint.len() > 1 {
                      let mut clients_with_duplicate_hint: Vec<Client> = clients_with_same_hint.clone();
                      acc.append(&mut clients_with_duplicate_hint);
                      acc
                  } else {
                      acc
                  }
              },
        );
}

fn as_client_and_hints(clients: Vec<Client>) -> Vec<ClientAndHint> {
    let mut client_and_hints = clients.iter()
        .map(|client| ClientAndHint {
            client: client.client_id.clone(),
            hint: client.hint.clone().unwrap(),
        })
        .collect::<Vec<_>>();

    client_and_hints.sort_by(|client_a, client_b| client_a.client.cmp(&client_b.client));
    return client_and_hints;
}

async fn check_guess(guess: String, game_id: &str, games: &Games) {
    println!("Guess: {}", guess);

    if let Ok(mut editable_games) = games.try_lock() {
        match editable_games.live_games.get_mut(game_id) {
            Some(game) => {
                let result = if guess.clone().to_lowercase() ==
                                      game.game_state.word_to_guess.as_ref().unwrap().to_lowercase() {
                    "correct"
                } else {
                    "incorrect"
                };

                let guess_result_message = json!({
                        "event": "guess_result",
                        "payload": {"result": result,
                                    "word": game.game_state.word_to_guess,
                                    "guess": guess
                       }
                    });

                let clients = game.clients.clone().into_iter().map(|(_, client)| client).collect::<Vec<_>>();
                for client in clients {
                    send_message(&client, &*guess_result_message.to_string()).await;

                    if client.hint == None {
                        let (unique_hinter_clients, duplicate_hinter_clients, _) =
                            uniques_and_duplicates(game.clients.clone());

                        let duplicates_to_guesser_message = json!({
                            "event": "all_hints",
                            "payload": {"duplicates": duplicate_hinter_clients,
                                        "hints": unique_hinter_clients
                                       }
                        });

                        send_message(&client, &*duplicates_to_guesser_message.to_string()).await;
                    }
                }
            }
            None => return // TODO Oh, no! Game not found! Return error?
        }
    } else {
        println!("Could not get lock for game state.");
    };
}

async fn remove_client(games: &Games, game_id: &str, client_id: &str) {
    println!("Removing client '{}' from game", client_id);
    if let Ok(mut editable_games) = games.try_lock() {
        match editable_games.live_games.get_mut(game_id) {
            Some(game) => {
                let clients = &mut game.clients;
                clients.remove(client_id);

                let game_state = &mut game.game_state;
                game_state.client_turns.retain(|c| c.client_id != client_id);
                println!("{} disconnected", client_id);

                for (_, client) in clients {
                    let user_quit_message = json!({
                        "event": "quit",
                        "payload": {"id": client_id}
                    });
                    send_message(client, &*user_quit_message.to_string()).await;
                }
                // TODO Remove game when last client disconnects?
            }
            None => return // TODO Oh, no! Game not found! Return error?
        }
    };
}
