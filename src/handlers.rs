use crate::{ws, Clients, Result};
use warp::Reply;

pub async fn ws_handler(session: String, ws: warp::ws::Ws, clients: Clients) -> Result<impl Reply> {
    println!("ws_handler");

    Ok(ws.on_upgrade(move |socket| ws::client_connection(session.clone(), socket, clients)))
}
