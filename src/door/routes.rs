use std::sync::Arc;

use axum::{Router, extract::State, routing::get};

use crate::{door, state::AppState};

macro_rules! door_handlers {
    ($($name:ident => $event:ident),*) => {
        $(
            async fn $name(State(state): State<Arc<AppState>>) {
                state.door.send(door::door::Event::$event).unwrap();
            }
        )*

        pub fn door_routes() -> Router<Arc<AppState>> {
            Router::new()
                $(.route(concat!("/", stringify!($name)), get($name)))*
        }
    };
}

door_handlers! {
    open    => Open,
    close   => Close,
    hold    => Hold,
    release => Release,
    lock    => Lock,
    unlock  => Unlock,
    calibrate  => Calibrate
}
