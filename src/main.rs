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
    use warp::test::WsClient;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    async fn expect_received(client: &mut WsClient, expected_message: &str) {
        let msg = client.recv().await.expect("recv");
        assert_eq!(msg.to_str(), Ok(expected_message));

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

    #[tokio::test]
    async fn created_game_contains_client_with_given_name() {
        let games= empty_games_state().await;

        start_game(&games, "user1").await;

        // TODO Can we do reading without lock?
        let current_games = games.lock().await;
        let game = current_games.live_games.get("1001").unwrap();
        let clients = game.clone().clients;
        for client in clients.values() {
            assert_eq!(client.username, "user1");
        }
    }

    #[tokio::test]
    async fn create_game_then_join_game_and_send_message() {
        let games= empty_games_state().await;

        let mut host_client = start_game(&games, "user1").await;

        let mut player_client= join_game(&games, "1001", "user2").await;

        host_client.send_text("hi from host").await;
        expect_received(&mut player_client, "hi from host").await;

        player_client.send_text("hi from player").await;
        expect_received(&mut host_client, "hi from player").await;
    }

    // TODO #1 join event is delivered
    // TODO #2 game start chooses word and notifies of roles
    // TODO #3 hint is stored in state
    // TODO #4 after last hint, duplicates notification is shown and guesser sees unique hints
    // TODO #5 guesser's guess is shown
    // TODO #6 select next guesser and word... -> #2

    // TODO #1.1 trying to join non-existent game gives clear error
    // TODO #2.1 can't start game with only one player
    // TODO #5.1 score is updated in state and notified to players

    // TODO #100.1 re-join with existing username
    // TODO #100.2 heartbeat to drop a player who has lost connection
}
