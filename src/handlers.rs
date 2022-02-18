use crate::{ws, Clients, Result};
use warp::Reply;

pub async fn new_game_handler(username: String, ws: warp::ws::Ws, clients: Clients) -> Result<impl Reply> {
    println!("new_game_handler user '{}'", username);
    let session = username.clone(); // TODO actually generate session

    Ok(ws.on_upgrade(move |socket| ws::client_connection(session.clone(), socket, clients)))
}

pub async fn join_game_handler(session: String, username :String, ws: warp::ws::Ws, clients: Clients) -> Result<impl Reply> {
    println!("join_game_handler user '{}' joining to session '{}'", username, session);

    // TODO validate session

    Ok(ws.on_upgrade(move |socket| ws::client_connection(session.clone(), socket, clients)))
}
