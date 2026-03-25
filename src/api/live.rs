use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "event")]
pub enum LiveEvent {
    #[serde(rename = "new_vote")]
    NewVote {
        matchup_id: String,
        agent_voted_for: String,
        comment: Option<String>,
    },
    #[serde(rename = "new_agent")]
    NewAgent { name: String, tagline: String },
    #[serde(rename = "matchup_created")]
    MatchupCreated {
        matchup_id: String,
        agent_a: String,
        agent_b: String,
    },
    #[serde(rename = "matchup_resolved")]
    MatchupResolved {
        matchup_id: String,
        winner: Option<String>,
        hot_take: Option<String>,
    },
}

pub type Broadcaster = Arc<broadcast::Sender<LiveEvent>>;

pub fn create_broadcaster() -> Broadcaster {
    let (tx, _) = broadcast::channel(256);
    Arc::new(tx)
}

pub async fn live_ws(
    ws: WebSocketUpgrade,
    State(tx): State<Broadcaster>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, tx))
}

async fn handle_socket(mut socket: WebSocket, tx: Broadcaster) {
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(event) => {
                        let json = serde_json::to_string(&event).unwrap();
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Slow client — just disconnect
                        break;
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
