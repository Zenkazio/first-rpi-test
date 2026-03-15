use std::sync::Arc;

use axum::{Router, extract::State, routing::get};

use crate::{door, state::AppState};

pub fn door_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/open", get(open))
        .route("/close", get(close))
        .route("/hold", get(hold))
        .route("/unhold", get(unhold))
}

async fn close(State(state): State<Arc<AppState>>) {
    state.door.send(door::door::Event::Unhold).unwrap();
    state.door.send(door::door::Event::Close).unwrap();
}

async fn open(State(state): State<Arc<AppState>>) {
    state.door.send(door::door::Event::Open).unwrap();
    state.door.send(door::door::Event::Hold).unwrap();
}

async fn hold(State(state): State<Arc<AppState>>) {
    state.door.send(door::door::Event::Hold).unwrap();
}
async fn unhold(State(state): State<Arc<AppState>>) {
    state.door.send(door::door::Event::Unhold).unwrap();
}
