use std::{collections::HashMap, convert::Infallible, sync::Arc};

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use warp::{Filter, Rejection, Reply, ws::Message};

use serde::Deserialize;

mod handlers;
mod ws;

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

    println!("Configuring websocket route");
    let ws_route = warp::path("ws");

    //ws/join/<session_id>/<username>
    let join_route = ws_route
        .and(warp::path("join"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::ws())
        .and(with_games(games.clone()))
        .and_then(handlers::join_game_handler);

    // let routes =
    //     new_route(&games)
    //     .or(join_route)
    //     .with(warp::cors().allow_any_origin());
    // println!("Starting server");
    // warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_games(games: Games) -> impl Filter<Extract = (Games,), Error = Infallible> + Clone {
    warp::any().map(move || games.clone())
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Websocket filter that echoes all messages back.
fn ws_echo() -> impl Filter<Extract = impl Reply, Error = Rejection> + Copy {
    warp::ws().map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| {
            // Just echo all messages back...
            let (tx, rx) = websocket.split();
            rx.inspect(|i| println!("ws recv: {:?}", i))
                .forward(tx)
                .map(|r| {})
        })
    })
}

#[derive(Deserialize)]
struct MyQuery {
    hello: String,
}

fn ws_route_with_path() -> impl Filter<Extract = impl Reply, Error = Rejection> + Copy {
    return warp::path("my-ws")
        .and(warp::query::<MyQuery>())
        .and(warp::ws())
        .map(|query: MyQuery, ws: warp::ws::Ws| {
            assert_eq!(query.hello, "world");

            ws.on_upgrade(|websocket| {
                let (tx, rx) = websocket.split();
                rx.inspect(|i| println!("ws recv: {:?}", i))
                    .forward(tx)
                    .map(|_| ())
            })
        });
}

// TODO how to test this?
// fn new_route() -> impl Filter<Extract = impl Reply, Error = Rejection> + Copy {
//     // TODO games should come as a parameter
//     let game_container = GameContainer { games_created: 0, live_games: HashMap::new() };
//     let games: Games = Arc::new(Mutex::new(game_container));
//
//     let ws_route = warp::path("ws");
//     // ws/new/<username>
//     let new_route = ws_route
//         .and(warp::path("new"))
//         .and(warp::path::param::<String>())
//         .and(warp::path::end())
//         .and(warp::ws())
//         .and(with_games(games.clone()))
//         .and_then(handlers::new_game_handler);
//
//    new_route
// }

#[cfg(test)]
mod tests {
    use futures::future;
    use warp::Filter;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }

    #[tokio::test]
    async fn test_ok_route_example() {
        let route = warp::ws()
            .map(|ws: warp::ws::Ws| {
                ws.on_upgrade(|_| future::ready(()))
            });

        let client = warp::test::ws()
            .handshake(route)
            .await
            .expect("handshake");
    }

    #[tokio::test]
    async fn test_dummy_route() {
        let route = warp::ws()
            .map(|ws: warp::ws::Ws| {
                ws.on_upgrade(|_| future::ready(()))
            });

        let client = warp::test::ws()
            .handshake(route)
            .await
            .expect("handshake");
    }

    #[tokio::test]
    async fn echo_ws_route() {
        let mut ws_client = warp::test::ws()
            .handshake(ws_echo())
            .await
            .expect("handshake");

        ws_client.send_text("hellox").await;
        let msg = ws_client.recv().await.expect("recv");
        assert_eq!(msg.to_str(), Ok("hellox"));
    }

    #[tokio::test]
    async fn ws_with_query() {
        let ws_filter = ws_route_with_path();

        warp::test::ws()
            .path("/my-ws?hello=world")
            .handshake(ws_filter)
            .await
            .expect("handshake");
    }
}
