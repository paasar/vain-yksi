use std::{collections::HashMap, convert::Infallible, sync::Arc};

use tokio::sync::{mpsc, Mutex};
use warp::{Filter, Rejection, Reply, ws::Message};

mod handlers;
mod ws;
mod words;

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub hint: Option<String>,
    pub username: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Debug, Clone)]
pub struct GameState {
    client_turns: Vec<Client>,
    word_to_guess: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub game_id: String,
    pub game_state: GameState,
    pub clients: HashMap<String, Client>,
}

#[derive(Debug, Clone)]
pub struct GameContainer {
    pub games_created: u32,
    pub live_games: HashMap<String, Game>,
    pub test_word: Option<String>,
}

type Games = Arc<Mutex<GameContainer>>;
type Result<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let game_container = GameContainer {
        games_created: 0,
        live_games: HashMap::new(),
        test_word: None,
    };
    let games: Games = Arc::new(Mutex::new(game_container));

    let static_files = warp::path::end()
        .and(warp::fs::dir("./static/"))
        .or(warp::path("assets").and(warp::fs::dir("./static/assets/")));

    println!("Configuring websocket routes");
    let routes =
        new_route(&games)
            .or(join_route(&games))
            .or(static_files)
            .with(warp::cors().allow_any_origin());

    println!("Starting server");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_games(games: Games) -> impl Filter<Extract=(Games, ), Error=Infallible> + Clone {
    warp::any().map(move || games.clone())
}

fn new_route(games: &Games) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    let ws_route = warp::path("ws");
    // ws/new/<username>
    let new_route = ws_route
        .and(warp::path("new"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::ws())
        .and(with_games(games.clone()))
        .and_then(handlers::new_game_handler);

    new_route
}

fn join_route(games: &Games) -> impl Filter<Extract=impl Reply, Error=Rejection> + Clone {
    let ws_route = warp::path("ws");
    // ws/join/<session_id>/<username>
    let join_route = ws_route
        .and(warp::path("join"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::ws())
        .and(with_games(games.clone()))
        .and_then(handlers::join_game_handler);

    join_route
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serde_json::json;
    use tokio::time::timeout;
    use warp::test::WsClient;
    use crate::ws::ClientIdAndName;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn new_game_msg() -> String {
        return json!({
            "event": "new_game",
            "payload": {"id": "1001"}
        }).to_string();
    }

    fn your_data_msg(username: &str) -> String {
        return json!({
            "event": "your_data",
            "payload": {"id": format!("{}_id", username),
                        "username": username}
        }).to_string();
    }

    fn other_players_msg(usernames: Vec<&str>) -> String {
        let other_players = usernames.into_iter()
            .map(|username| ClientIdAndName {
                id: format!("{}_id", username),
                username: String::from(username),
            })
            .collect::<Vec<_>>();
        return json!({
            "event": "other_players",
            "payload": other_players
        }).to_string();
    }

    async fn assert_message(client: &mut WsClient, expected_message: &str) {
        let msg = client.recv().await.expect("recv");
        assert_eq!(msg.to_str(), Ok(expected_message));

        return;
    }

    async fn expect_received(client: &mut WsClient, expected_message: &str) {
        if let Err(_) = timeout(Duration::from_secs(2),
                                assert_message(client, expected_message)).await {
            assert!(false, "Did not finish in time!");
        }

        return;
    }

    async fn create_empty_games_state() -> Games {
        let game_container = GameContainer {
            games_created: 0,
            live_games: HashMap::new(),
            test_word: Some(String::from("testisana")),
        };
        return Arc::new(Mutex::new(game_container));
    }

    async fn start_game(games: &Games, username: &str) -> WsClient {
        let route = new_route(games);

        return warp::test::ws()
            .path(&*format!("/ws/new/{}", username))
            .handshake(route)
            .await
            .expect("handshake");
    }

    async fn join_game(games: &Games, game_id: &str, username: &str) -> WsClient {
        let route = join_route(games);

        return warp::test::ws()
            .path(&*format!("/ws/join/{}/{}", game_id, username))
            .handshake(route)
            .await
            .expect("handshake");
    }

    // Case #1
    #[tokio::test]
    async fn new_game_creator_is_sent_the_game_id() {
        let games = create_empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;

        expect_received(&mut host_client, &*new_game_msg()).await;
        expect_received(&mut host_client, &*your_data_msg("user1")).await;
    }

    // Case #2
    #[tokio::test]
    async fn join_event_is_delivered_to_existing_players() {
        let games = create_empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;
        expect_received(&mut host_client, &*new_game_msg().to_string()).await;
        expect_received(&mut host_client, &*your_data_msg("user1")).await;

        let mut second_client = join_game(&games, "1001", "user2").await;
        let user2_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user2_id",
                "username": "user2"
            }
        });
        expect_received(&mut host_client, &*user2_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*other_players_msg(vec!("user1"))).await;
        expect_received(&mut second_client, &*your_data_msg("user2")).await;

        let mut third_client = join_game(&games, "1001", "user3").await;
        let user3_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user3_id",
                "username": "user3"
            }
        });
        expect_received(&mut host_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*other_players_msg(vec!("user1", "user2"))).await;
        expect_received(&mut third_client, &*your_data_msg("user3")).await;

        if let Ok(current_games) = games.try_lock() {
            let game = current_games.live_games.get("1001").unwrap();
            let clients = game.clone().clients;
            assert_eq!(3, clients.len());
        } else {
            assert!(false, "Could not get lock to assert game state.");
        };
    }

    // Case #3
    #[tokio::test]
    async fn staring_game_chooses_word_and_notifies_roles() {
        let games = create_empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;
        expect_received(&mut host_client, &*new_game_msg().to_string()).await;
        expect_received(&mut host_client, &*your_data_msg("user1")).await;

        let mut second_client = join_game(&games, "1001", "user2").await;
        let user2_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user2_id",
                "username": "user2"
            }
        });
        expect_received(&mut host_client, &*user2_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*other_players_msg(vec!("user1"))).await;
        expect_received(&mut second_client, &*your_data_msg("user2")).await;

        let mut third_client = join_game(&games, "1001", "user3").await;
        let user3_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user3_id",
                "username": "user3"
            }
        });
        expect_received(&mut host_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*other_players_msg(vec!("user1", "user2"))).await;
        expect_received(&mut third_client, &*your_data_msg("user3")).await;

        // ---- Setup done ----

        let start_next_round_msg = json!({
            "action": {"start_next_round": true}
        });
        host_client.send(Message::text(start_next_round_msg.to_string())).await;
        let new_round_guesser_msg = json!({
            "event": "new_round",
            "payload": {"role": "guesser"}
        });
        expect_received(&mut host_client, &*new_round_guesser_msg.to_string()).await;

        let new_round_hinter_msg = json!({
            "event": "new_round",
            "payload": {"role": "hinter",
                        "word": "testisana",
                        "guesser": "user1_id"}
        });
        expect_received(&mut second_client, &*new_round_hinter_msg.to_string()).await;
        expect_received(&mut third_client, &*new_round_hinter_msg.to_string()).await;

        if let Ok(current_games) = games.try_lock() {
            let game = current_games.live_games.get("1001").unwrap();
            match game.clone().game_state.word_to_guess {
                // TODO Assert that all hints are None
                Some(word_to_guess) => assert_eq!("testisana", word_to_guess),
                None => assert!(false, "No word to guess in state.")
            }
        } else {
            assert!(false, "Cloud not get lock to assert game state.");
        };
    }

    // Case #4 & #5
    #[tokio::test]
    async fn sent_hints_are_stored_and_after_last_hint_result_are_sent() {
        let games = create_empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;
        expect_received(&mut host_client, &*new_game_msg().to_string()).await;
        expect_received(&mut host_client, &*your_data_msg("user1")).await;

        let mut second_client = join_game(&games, "1001", "user2").await;
        let user2_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user2_id",
                "username": "user2"
            }
        });
        expect_received(&mut host_client, &*user2_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*other_players_msg(vec!("user1"))).await;
        expect_received(&mut second_client, &*your_data_msg("user2")).await;

        let mut third_client = join_game(&games, "1001", "user3").await;
        let user3_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user3_id",
                "username": "user3"
            }
        });
        expect_received(&mut host_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*other_players_msg(vec!("user1", "user2"))).await;
        expect_received(&mut third_client, &*your_data_msg("user3")).await;

        let mut fourth_client = join_game(&games, "1001", "user4").await;
        let user4_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user4_id",
                "username": "user4"
            }
        });
        expect_received(&mut host_client, &*user4_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user4_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*user4_joined_msg.to_string()).await;
        expect_received(&mut fourth_client, &*other_players_msg(vec!("user1", "user2", "user3"))).await;
        expect_received(&mut fourth_client, &*your_data_msg("user4")).await;

        let start_next_round_msg = json!({
            "action": {"start_next_round": true}
        });
        host_client.send(Message::text(start_next_round_msg.to_string())).await;
        let new_round_guesser_msg = json!({
            "event": "new_round",
            "payload": {"role": "guesser"}
        });
        expect_received(&mut host_client, &*new_round_guesser_msg.to_string()).await;

        let new_round_hinter_msg = json!({
            "event": "new_round",
            "payload": {"role": "hinter",
                        "word": "testisana",
                        "guesser": "user1_id"}
        });
        expect_received(&mut second_client, &*new_round_hinter_msg.to_string()).await;
        expect_received(&mut third_client, &*new_round_hinter_msg.to_string()).await;
        expect_received(&mut fourth_client, &*new_round_hinter_msg.to_string()).await;

        // ---- Setup done ----

        let hint2_msg = json!({
            "action": {"hint": "vinkki2"}
        });
        second_client.send(Message::text(hint2_msg.to_string())).await;

        let hint_received_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user2_id"}
        });
        expect_received(&mut host_client, &*hint_received_msg.to_string()).await;
        expect_received(&mut third_client, &*hint_received_msg.to_string()).await;
        expect_received(&mut fourth_client, &*hint_received_msg.to_string()).await;

        if let Ok(current_games) = games.try_lock() {
            let game = current_games.live_games.get("1001").unwrap();
            let clients = game.clone().clients;
            assert_eq!(Some(String::from("vinkki2")), clients.get("user2_id").unwrap().hint);
        } else {
            println!("Cloud not get lock to assert game state.");
        };

        // Case #5 Add more hints, after last hint, hints and duplicates notification is sent and
        // guesser sees only unique hints
        let hint3_msg = json!({
            "action": {"hint": "vinkki3"}
        });
        third_client.send(Message::text(hint3_msg.to_string())).await;

        let hint_received_from3_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user3_id"}
        });
        expect_received(&mut host_client, &*hint_received_from3_msg.to_string()).await;
        expect_received(&mut second_client, &*hint_received_from3_msg.to_string()).await;
        expect_received(&mut fourth_client, &*hint_received_from3_msg.to_string()).await;

        // Use same hint as user3 to cause a duplicate hint
        fourth_client.send(Message::text(hint3_msg.to_string())).await;

        let hint_received_from4_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user4_id"}
        });
        expect_received(&mut host_client, &*hint_received_from4_msg.to_string()).await;
        expect_received(&mut second_client, &*hint_received_from4_msg.to_string()).await;
        expect_received(&mut third_client, &*hint_received_from4_msg.to_string()).await;

        let hints_to_guesser_msg = json!({
            "event": "all_hints_to_guesser",
            "payload": {"hints": [{"client": "user2_id",
                                   "hint": "vinkki2"
                                  }],
                        "usersWithDuplicates": ["user3_id", "user4_id"]
                       }
        });
        expect_received(&mut host_client, &*hints_to_guesser_msg.to_string()).await;

        let hints_to_hinters_msg = json!({
            "event": "all_hints",
            "payload": {"duplicates": [{"client": "user3_id",
                                        "hint": "vinkki3"
                                       },
                                       {"client": "user4_id",
                                        "hint": "vinkki3"
                                       }],
                        "hints": [{"client": "user2_id",
                                   "hint": "vinkki2"
                                  }]
                       }
        });
        expect_received(&mut second_client, &*hints_to_hinters_msg.to_string()).await;
        expect_received(&mut third_client, &*hints_to_hinters_msg.to_string()).await;
        expect_received(&mut fourth_client, &*hints_to_hinters_msg.to_string()).await;
    }

    // Case #6.1
    #[tokio::test]
    async fn correct_guess_is_given() {
        let games = create_empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;
        expect_received(&mut host_client, &*new_game_msg().to_string()).await;
        expect_received(&mut host_client, &*your_data_msg("user1")).await;

        let mut second_client = join_game(&games, "1001", "user2").await;
        let user2_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user2_id",
                "username": "user2"
            }
        });
        expect_received(&mut host_client, &*user2_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*other_players_msg(vec!("user1"))).await;
        expect_received(&mut second_client, &*your_data_msg("user2")).await;

        let mut third_client = join_game(&games, "1001", "user3").await;
        let user3_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user3_id",
                "username": "user3"
            }
        });
        expect_received(&mut host_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*other_players_msg(vec!("user1", "user2"))).await;
        expect_received(&mut third_client, &*your_data_msg("user3")).await;

        let mut fourth_client = join_game(&games, "1001", "user4").await;
        let user4_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user4_id",
                "username": "user4"
            }
        });
        expect_received(&mut host_client, &*user4_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user4_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*user4_joined_msg.to_string()).await;
        expect_received(&mut fourth_client, &*other_players_msg(vec!("user1", "user2", "user3"))).await;
        expect_received(&mut fourth_client, &*your_data_msg("user4")).await;

        let start_next_round_msg = json!({
            "action": {"start_next_round": true}
        });
        host_client.send(Message::text(start_next_round_msg.to_string())).await;
        let new_round_guesser_msg = json!({
            "event": "new_round",
            "payload": {"role": "guesser"}
        });
        expect_received(&mut host_client, &*new_round_guesser_msg.to_string()).await;

        let new_round_hinter_msg = json!({
            "event": "new_round",
            "payload": {"role": "hinter",
                        "word": "testisana",
                        "guesser": "user1_id"}
        });
        expect_received(&mut second_client, &*new_round_hinter_msg.to_string()).await;
        expect_received(&mut third_client, &*new_round_hinter_msg.to_string()).await;
        expect_received(&mut fourth_client, &*new_round_hinter_msg.to_string()).await;

        let hint2_msg = json!({
            "action": {"hint": "vinkki2"}
        });
        second_client.send(Message::text(hint2_msg.to_string())).await;

        let hint_received_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user2_id"}
        });
        expect_received(&mut host_client, &*hint_received_msg.to_string()).await;
        expect_received(&mut third_client, &*hint_received_msg.to_string()).await;
        expect_received(&mut fourth_client, &*hint_received_msg.to_string()).await;

        if let Ok(current_games) = games.try_lock() {
            let game = current_games.live_games.get("1001").unwrap();
            let clients = game.clone().clients;
            assert_eq!(Some(String::from("vinkki2")), clients.get("user2_id").unwrap().hint);
        } else {
            println!("Cloud not get lock to assert game state.");
        };

        // Case #5 Add more hints, after last hint, hints and duplicates notification is sent and
        // guesser sees only unique hints
        let hint3_msg = json!({
            "action": {"hint": "vinkki3"}
        });
        third_client.send(Message::text(hint3_msg.to_string())).await;

        let hint_received_from3_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user3_id"}
        });
        expect_received(&mut host_client, &*hint_received_from3_msg.to_string()).await;
        expect_received(&mut second_client, &*hint_received_from3_msg.to_string()).await;
        expect_received(&mut fourth_client, &*hint_received_from3_msg.to_string()).await;

        // Use same hint as user3 to cause a duplicate hint
        fourth_client.send(Message::text(hint3_msg.to_string())).await;

        let hint_received_from4_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user4_id"}
        });
        expect_received(&mut host_client, &*hint_received_from4_msg.to_string()).await;
        expect_received(&mut second_client, &*hint_received_from4_msg.to_string()).await;
        expect_received(&mut third_client, &*hint_received_from4_msg.to_string()).await;

        let hints_to_guesser_msg = json!({
            "event": "all_hints_to_guesser",
            "payload": {"hints": [{"client": "user2_id",
                                   "hint": "vinkki2"
                                  }],
                        "usersWithDuplicates": ["user3_id", "user4_id"]
                       }
        });
        expect_received(&mut host_client, &*hints_to_guesser_msg.to_string()).await;

        let hints_to_hinters_msg = json!({
            "event": "all_hints",
            "payload": {"duplicates": [{"client": "user3_id",
                                        "hint": "vinkki3"
                                       },
                                       {"client": "user4_id",
                                        "hint": "vinkki3"
                                       }],
                        "hints": [{"client": "user2_id",
                                   "hint": "vinkki2"
                                  }]
                       }
        });
        expect_received(&mut second_client, &*hints_to_hinters_msg.to_string()).await;
        expect_received(&mut third_client, &*hints_to_hinters_msg.to_string()).await;
        expect_received(&mut fourth_client, &*hints_to_hinters_msg.to_string()).await;

        // ---- Setup done ----

        let correct_guess_msg = json!({
            "action": {"guess": "testisana"}
        });
        host_client.send(Message::text(correct_guess_msg.to_string())).await;

        let correct_result_msg = json!({
            "event": "guess_result",
            "payload": {"result": "correct",
                         "word": "testisana",
                         "guess": "testisana"
                       }
        });

        expect_received(&mut host_client, &*correct_result_msg.to_string()).await;
        expect_received(&mut second_client, &*correct_result_msg.to_string()).await;
        expect_received(&mut third_client, &*correct_result_msg.to_string()).await;
        expect_received(&mut fourth_client, &*correct_result_msg.to_string()).await;

        expect_received(&mut host_client, &*hints_to_hinters_msg.to_string()).await;
    }

    // Case #6.2
    #[tokio::test]
    async fn incorrect_guess_is_given() {
        let games = create_empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;
        expect_received(&mut host_client, &*new_game_msg().to_string()).await;
        expect_received(&mut host_client, &*your_data_msg("user1")).await;

        let mut second_client = join_game(&games, "1001", "user2").await;
        let user2_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user2_id",
                "username": "user2"
            }
        });
        expect_received(&mut host_client, &*user2_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*other_players_msg(vec!("user1"))).await;
        expect_received(&mut second_client, &*your_data_msg("user2")).await;

        let mut third_client = join_game(&games, "1001", "user3").await;
        let user3_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user3_id",
                "username": "user3"
            }
        });
        expect_received(&mut host_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*other_players_msg(vec!("user1", "user2"))).await;
        expect_received(&mut third_client, &*your_data_msg("user3")).await;

        let mut fourth_client = join_game(&games, "1001", "user4").await;
        let user4_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user4_id",
                "username": "user4"
            }
        });
        expect_received(&mut host_client, &*user4_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user4_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*user4_joined_msg.to_string()).await;
        expect_received(&mut fourth_client, &*other_players_msg(vec!("user1", "user2", "user3"))).await;
        expect_received(&mut fourth_client, &*your_data_msg("user4")).await;

        let start_next_round_msg = json!({
            "action": {"start_next_round": true}
        });
        host_client.send(Message::text(start_next_round_msg.to_string())).await;
        let new_round_guesser_msg = json!({
            "event": "new_round",
            "payload": {"role": "guesser"}
        });
        expect_received(&mut host_client, &*new_round_guesser_msg.to_string()).await;

        let new_round_hinter_msg = json!({
            "event": "new_round",
            "payload": {"role": "hinter",
                        "word": "testisana",
                        "guesser": "user1_id"}
        });
        expect_received(&mut second_client, &*new_round_hinter_msg.to_string()).await;
        expect_received(&mut third_client, &*new_round_hinter_msg.to_string()).await;
        expect_received(&mut fourth_client, &*new_round_hinter_msg.to_string()).await;

        let hint2_msg = json!({
            "action": {"hint": "vinkki2"}
        });
        second_client.send(Message::text(hint2_msg.to_string())).await;

        let hint_received_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user2_id"}
        });
        expect_received(&mut host_client, &*hint_received_msg.to_string()).await;
        expect_received(&mut third_client, &*hint_received_msg.to_string()).await;
        expect_received(&mut fourth_client, &*hint_received_msg.to_string()).await;

        if let Ok(current_games) = games.try_lock() {
            let game = current_games.live_games.get("1001").unwrap();
            let clients = game.clone().clients;
            assert_eq!(Some(String::from("vinkki2")), clients.get("user2_id").unwrap().hint);
        } else {
            println!("Cloud not get lock to assert game state.");
        };

        // Case #5 Add more hints, after last hint, hints and duplicates notification is sent and
        // guesser sees only unique hints
        let hint3_msg = json!({
            "action": {"hint": "vinkki3"}
        });
        third_client.send(Message::text(hint3_msg.to_string())).await;

        let hint_received_from3_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user3_id"}
        });
        expect_received(&mut host_client, &*hint_received_from3_msg.to_string()).await;
        expect_received(&mut second_client, &*hint_received_from3_msg.to_string()).await;
        expect_received(&mut fourth_client, &*hint_received_from3_msg.to_string()).await;

        // Use same hint as user3 to cause a duplicate hint
        fourth_client.send(Message::text(hint3_msg.to_string())).await;

        let hint_received_from4_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user4_id"}
        });
        expect_received(&mut host_client, &*hint_received_from4_msg.to_string()).await;
        expect_received(&mut second_client, &*hint_received_from4_msg.to_string()).await;
        expect_received(&mut third_client, &*hint_received_from4_msg.to_string()).await;

        let hints_to_guesser_msg = json!({
            "event": "all_hints_to_guesser",
            "payload": {"hints": [{"client": "user2_id",
                                   "hint": "vinkki2"
                                  }],
                        "usersWithDuplicates": ["user3_id", "user4_id"]
                       }
        });
        expect_received(&mut host_client, &*hints_to_guesser_msg.to_string()).await;

        let hints_to_hinters_msg = json!({
            "event": "all_hints",
            "payload": {"duplicates": [{"client": "user3_id",
                                        "hint": "vinkki3"
                                       },
                                       {"client": "user4_id",
                                        "hint": "vinkki3"
                                       }],
                        "hints": [{"client": "user2_id",
                                   "hint": "vinkki2"
                                  }]
                       }
        });
        expect_received(&mut second_client, &*hints_to_hinters_msg.to_string()).await;
        expect_received(&mut third_client, &*hints_to_hinters_msg.to_string()).await;
        expect_received(&mut fourth_client, &*hints_to_hinters_msg.to_string()).await;

        // ---- Setup done ----

        let incorrect_guess_msg = json!({
            "action": {"guess": "wrong"}
        });
        host_client.send(Message::text(incorrect_guess_msg.to_string())).await;

        let incorrect_result_msg = json!({
            "event": "guess_result",
            "payload": { "result": "incorrect",
                         "word": "testisana",
                         "guess": "wrong"
                       }
        });

        expect_received(&mut host_client, &*incorrect_result_msg.to_string()).await;
        expect_received(&mut second_client, &*incorrect_result_msg.to_string()).await;
        expect_received(&mut third_client, &*incorrect_result_msg.to_string()).await;
        expect_received(&mut fourth_client, &*incorrect_result_msg.to_string()).await;

        expect_received(&mut host_client, &*hints_to_hinters_msg.to_string()).await;
    }

    // Case #7
    #[tokio::test]
    async fn requesting_new_round_gives_word_and_notifies_roles() {
        let games = create_empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;
        expect_received(&mut host_client, &*new_game_msg().to_string()).await;
        expect_received(&mut host_client, &*your_data_msg("user1")).await;

        let mut second_client = join_game(&games, "1001", "user2").await;
        let user2_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user2_id",
                "username": "user2"
            }
        });
        expect_received(&mut host_client, &*user2_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*other_players_msg(vec!("user1"))).await;
        expect_received(&mut second_client, &*your_data_msg("user2")).await;

        let mut third_client = join_game(&games, "1001", "user3").await;
        let user3_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user3_id",
                "username": "user3"
            }
        });
        expect_received(&mut host_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*other_players_msg(vec!("user1", "user2"))).await;
        expect_received(&mut third_client, &*your_data_msg("user3")).await;

        let start_next_round_msg = json!({
            "action": {"start_next_round": true}
        });
        host_client.send(Message::text(start_next_round_msg.to_string())).await;
        let new_round_guesser_msg = json!({
            "event": "new_round",
            "payload": {"role": "guesser"}
        });
        expect_received(&mut host_client, &*new_round_guesser_msg.to_string()).await;

        let new_round_hinter_msg = json!({
            "event": "new_round",
            "payload": {"role": "hinter",
                        "word": "testisana",
                        "guesser": "user1_id"}
        });
        expect_received(&mut second_client, &*new_round_hinter_msg.to_string()).await;
        expect_received(&mut third_client, &*new_round_hinter_msg.to_string()).await;

        let hint3_msg = json!({
            "action": {"hint": "vinkki3"}
        });
        third_client.send(Message::text(hint3_msg.to_string())).await;

        let hint_received_from3_msg = json!({
            "event": "hint_received",
            "payload": {"client": "user3_id"}
        });
        expect_received(&mut host_client, &*hint_received_from3_msg.to_string()).await;
        expect_received(&mut second_client, &*hint_received_from3_msg.to_string()).await;

        if let Ok(current_games) = games.try_lock() {
            let game = current_games.live_games.get("1001").unwrap();
            match game.clone().game_state.word_to_guess {
                Some(word_to_guess) => assert_eq!("testisana", word_to_guess),
                None => assert!(false, "No word to guess in state.")
            }
        } else {
            assert!(false, "Cloud not get lock to assert game state.")
        };

        // ---- Setup done ----

        host_client.send(Message::text(start_next_round_msg.to_string())).await;

        let new_round_hinter2_msg = json!({
            "event": "new_round",
            "payload": {"role": "hinter",
                        "word": "testisana",
                        "guesser": "user2_id"}
        });
        expect_received(&mut host_client, &*new_round_hinter2_msg.to_string()).await;
        expect_received(&mut second_client, &*new_round_guesser_msg.to_string()).await;
        expect_received(&mut third_client, &*new_round_hinter2_msg.to_string()).await;

        // Assert that all hints have been reset
        if let Ok(current_games) = games.try_lock() {
            let game = current_games.live_games.get("1001").unwrap();
            for (_, client) in game.clone().clients {
                assert_eq!(None, client.hint)
            }
        } else {
            assert!(false, "Cloud not get lock to assert game state.")
        };
    }

    // Case #8
    #[tokio::test]
    async fn skip_word_and_retain_roles() {
        let games = create_empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;
        expect_received(&mut host_client, &*new_game_msg().to_string()).await;
        expect_received(&mut host_client, &*your_data_msg("user1")).await;

        let mut second_client = join_game(&games, "1001", "user2").await;
        let user2_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user2_id",
                "username": "user2"
            }
        });
        expect_received(&mut host_client, &*user2_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*other_players_msg(vec!("user1"))).await;
        expect_received(&mut second_client, &*your_data_msg("user2")).await;

        let start_next_round_msg = json!({
            "action": {"start_next_round": true}
        });
        host_client.send(Message::text(start_next_round_msg.to_string())).await;
        let new_round_guesser_msg = json!({
            "event": "new_round",
            "payload": {"role": "guesser"}
        });
        expect_received(&mut host_client, &*new_round_guesser_msg.to_string()).await;

        let new_round_hinter_msg = json!({
            "event": "new_round",
            "payload": {"role": "hinter",
                        "word": "testisana",
                        "guesser": "user1_id"}
        });
        expect_received(&mut second_client, &*new_round_hinter_msg.to_string()).await;

        // ---- Setup done ----

        if let Ok(mut current_games) = games.try_lock() {
            current_games.test_word = Some(String::from("sanatesti"));
        } else {
            assert!(false, "Cloud not get lock to change game state.")
        }

        let skip_word_msg = json!({
            "action": {"skip_word": true}
        });
        host_client.send(Message::text(skip_word_msg.to_string())).await;

        let new_round_guesser_msg = json!({
            "event": "new_round",
            "payload": {"role": "guesser"}
        });
        expect_received(&mut host_client, &*new_round_guesser_msg.to_string()).await;

        let new_round_hinter_with_new_word_msg = json!({
            "event": "new_round",
            "payload": {"role": "hinter",
                        "word": "sanatesti",
                        "guesser": "user1_id"}
        });
        expect_received(&mut second_client, &*new_round_hinter_with_new_word_msg.to_string()).await;
    }

    // Case #9
    #[tokio::test]
    async fn player_quit_is_informed_to_all_others() {
        let games = create_empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;
        expect_received(&mut host_client, &*new_game_msg().to_string()).await;
        expect_received(&mut host_client, &*your_data_msg("user1")).await;

        let mut second_client = join_game(&games, "1001", "user2").await;
        let user2_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user2_id",
                "username": "user2"
            }
        });
        expect_received(&mut host_client, &*user2_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*other_players_msg(vec!("user1"))).await;
        expect_received(&mut second_client, &*your_data_msg("user2")).await;

        let mut third_client = join_game(&games, "1001", "user3").await;
        let user3_joined_msg = json!({
            "event": "join",
            "payload": {
                "id": "user3_id",
                "username": "user3"
            }
        });
        expect_received(&mut host_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut third_client, &*other_players_msg(vec!("user1", "user2"))).await;
        expect_received(&mut third_client, &*your_data_msg("user3")).await;

        // ---- Setup done ----

        drop(third_client);

        let user_quit_msg = json!({
            "event": "quit",
            "payload": {"id": "user3_id"}
        });

        expect_received(&mut host_client, &*user_quit_msg.to_string()).await;
        expect_received(&mut second_client, &*user_quit_msg.to_string()).await;
    }

    // Nice to have
    // TODO Case #2.1 trying to join non-existent game gives clear error
    // TODO Case #2.2 join after game is started
    // TODO Case #3.1 can't start game with only one player
    // TODO Case #6.3 score is updated in state and notified to players
    // TODO Case #9.1 player quit event (as guesser)
    // TODO Case #9.2 player quit event (as hint giver)

    // Under consideration
    // TODO Case #100 "user NN is typing"
    // TODO Case #101 re-join with existing username
    // TODO Case #102 heartbeat to drop a player who has lost connection
    // TODO Case #104 test multiple concurrent games
}
