use crate::{ws, Games, Result};
use warp::Reply;
use urlencoding;
use crate::words::WordGenerator;

pub async fn new_game_handler(username: String,
                              ws: warp::ws::Ws,
                              games: Games,
                              word_generator: impl WordGenerator) -> Result<impl Reply> {
    println!("new_game_handler user '{}'", username);

    Ok(ws.on_upgrade(move |socket| ws::new_game(
        urldecode_username(username.clone()),
        socket,
        games,
        word_generator)))
}

pub async fn join_game_handler(session: String,
                               username :String,
                               ws: warp::ws::Ws,
                               games: Games,
                               word_generator: impl WordGenerator) -> Result<impl Reply> {
    println!("join_game_handler user '{}' joining to session '{}'", username, session);

    // TODO validate session

    Ok(ws.on_upgrade(move |socket| ws::join_game(
        urldecode_username(username.clone()),
        socket,
        games,
        word_generator,
        session.clone())))
}

fn urldecode_username(username: String) -> String {
    urlencoding::decode(&username).expect("UTF-8").to_string()
}
