use std::{collections::HashMap, convert::Infallible, sync::Arc};

use tokio::sync::{mpsc, Mutex};
use warp::{Filter, Rejection, Reply, ws::Message};

mod handlers;
mod ws;
mod words;

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: String,
    pub username: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Debug, Clone)]
pub struct GameState {
    word_to_guess: Option<String>
}

#[derive(Debug, Clone)]
pub struct Game {
    pub game_id: String,
    pub game_state: GameState,
    pub clients: HashMap<String, Client>
}

#[derive(Debug, Clone)]
pub struct GameContainer {
    pub games_created: u32,
    pub live_games: HashMap<String, Game>
}

type Games = Arc<Mutex<GameContainer>>;
type Result<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {
    let game_container = GameContainer { games_created: 0, live_games: HashMap::new() };
    let games: Games = Arc::new(Mutex::new(game_container));

    println!("Configuring websocket routes");
    let routes =
        new_route(&games)
        .or(join_route(&games))
        .with(warp::cors().allow_any_origin());

    println!("Starting server");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_games(games: Games) -> impl Filter<Extract = (Games,), Error = Infallible> + Clone {
    warp::any().map(move || games.clone())
}

fn new_route(games: &Games) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
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

fn join_route(games: &Games) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
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

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

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

    async fn empty_games_state() -> Games {
        let game_container = GameContainer { games_created: 0, live_games: HashMap::new() };
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

    // TODO Case #1 when game is created send game id

    // Case #2
    #[tokio::test]
    async fn join_event_is_delivered_to_existing_players() {
        let games= empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;

        let mut second_client = join_game(&games, "1001", "user2").await;
        // TODO Add player id to playload? -> Can't compare as a plain string then.
        let user2_joined_msg = json!({
            "event": "join",
            "payload": {"name": "user2"}
        });
        expect_received(&mut host_client, &*user2_joined_msg.to_string()).await;

        join_game(&games, "1001", "user3").await;
        let user3_joined_msg = json!({
            "event": "join",
            "payload": {"name": "user3"}
        });
        expect_received(&mut host_client, &*user3_joined_msg.to_string()).await;
        expect_received(&mut second_client, &*user3_joined_msg.to_string()).await;

        let current_games = games.lock().await;
        let game = current_games.live_games.get("1001").unwrap();
        let clients = game.clone().clients;
        assert_eq!(3, clients.len());
    }

    // TODO Case #3 game start chooses word and notifies of roles
    // TODO Case #4 hint is stored in state
    // TODO Case #5 after last hint, duplicates notification is shown and guesser sees unique hints
    // TODO Case #6 guesser's guess is shown
    // TODO Case #7 select next guesser and word... -> #2

    // Nice to have
    // TODO Case #2.1 trying to join non-existent game gives clear error
    // TODO Case #2.2 player quit event
    // TODO Case #3.1 can't start game with only one player
    // TODO Case #6.1 score is updated in state and notified to players

    // Under consideration
    // TODO Case #100.1 re-join with existing username
    // TODO Case #100.2 heartbeat to drop a player who has lost connection
}
