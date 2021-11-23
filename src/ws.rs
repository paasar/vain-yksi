use crate::{Client, Clients};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};

pub async fn client_connection(session: String, ws: WebSocket, clients: Clients) {
    println!("establishing client connection... {:?}", ws);
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            println!("error sending websocket msg: {}", e);
        }
    }));

    let session_id = session.clone();
    let uuid = Uuid::new_v4().to_simple().to_string();
    let new_client = Client {
        client_id: uuid.clone(),
        session: session_id.clone(),
        sender: Some(client_sender),
    };
    clients.lock().await.insert(uuid.clone(), new_client);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                println!("error receiving message for id {}): {}", uuid.clone(), e);
                break;
            }
        };
        client_msg(&uuid, session_id.clone(), msg, &clients).await;
    }
    clients.lock().await.remove(&uuid);
    println!("{} disconnected", uuid);
}

async fn client_msg(client_id: &str, session: String, msg: Message, clients: &Clients) {
    println!("received message from {}: {:?}", client_id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };
    let locked = clients.lock().await;
    match locked.get(client_id) {
        Some(v) => {
            if let Some(sender) = &v.sender {
                if message == "ping" || message == "ping\n" {
                    println!("sending pong");
                    let _ = sender.send(Ok(Message::text("pong")));
                }
            }
        }
        None => return,
    }

    let _ = locked.values()
        .filter(|client| client.session == session && client.client_id != client_id)
        .for_each(|client| if let Some(sender) = &client.sender {
            println!("{} sending '{}' to {}", client_id, message, &client.client_id);
            let _ = sender.send(Ok(Message::text(format!("{} from {}", message, &client.client_id))));
        });
    return;
}
