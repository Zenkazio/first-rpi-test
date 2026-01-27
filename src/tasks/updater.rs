use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use humanize_duration::prelude::DurationExt;
use tokio::time::sleep;

use crate::{state::AppState, ws::messages::ServerMsg};

pub async fn status_update(state: Arc<AppState>) {
    let start = Instant::now();
    loop {
        sleep(Duration::from_secs(1)).await;
        let _ = state.tx.send(ServerMsg::CounterUpdate {
            value: start
                .elapsed()
                .human(humanize_duration::Truncate::Second)
                .to_string(),
        });
    }
}
