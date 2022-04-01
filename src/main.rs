use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use warp::{ws::Message, Filter, Rejection};
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

    // ws/new/<username>
    let new_route = ws_route
        .and(warp::path("new"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::ws())
        .and(with_games(games.clone()))
        .and_then(handlers::new_game_handler);

    //ws/join/<session_id>/<username>
    let join_route = ws_route
        .and(warp::path("join"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(warp::ws())
        .and(with_games(games.clone()))
        .and_then(handlers::join_game_handler);

    let routes =
        new_route
        .or(join_route)
        .with(warp::cors().allow_any_origin());
    println!("Starting server");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_games(games: Games) -> impl Filter<Extract = (Games,), Error = Infallible> + Clone {
    warp::any().map(move || games.clone())
}
