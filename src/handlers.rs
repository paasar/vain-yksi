use crate::{ws, Games, Result};
use warp::Reply;
use urlencoding;

pub async fn new_game_handler(username: String, ws: warp::ws::Ws, games: Games) -> Result<impl Reply> {
    println!("new_game_handler user '{}'", username);

    Ok(ws.on_upgrade(move |socket| ws::new_game(
        urldecode_username(username.clone()),
        socket,
        games)))
}

pub async fn join_game_handler(session: String, username :String, ws: warp::ws::Ws, games: Games) -> Result<impl Reply> {
    println!("join_game_handler user '{}' joining to session '{}'", username, session);

    // TODO validate session

    Ok(ws.on_upgrade(move |socket| ws::join_game(
        urldecode_username(username.clone()),
        socket,
        games,
        session.clone())))
}

fn urldecode_username(username: String) -> String {
    urlencoding::decode(&username).expect("UTF-8").to_string()
}
