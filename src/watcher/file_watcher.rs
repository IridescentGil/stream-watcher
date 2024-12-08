use crate::watcher::Streams;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc::Sender;

use super::StreamConfig;

pub async fn file_watcher(
    file_watcher_twitch_websocket_sender: Sender<u32>,
    file_watcher_event_handler_sender: Sender<StreamConfig>,
    _streams_path: &Path,
    streams: &Arc<Mutex<Streams>>,
) {
    let streamers = streams
        .lock()
        .as_ref()
        .expect("Mutex lock poisoned")
        .names
        .clone();
    let streamers = streamers.iter();
    let quality_iter = streams
        .lock()
        .as_ref()
        .expect("Mutex lock poisoned")
        .quality_overides
        .clone();
    let mut quality_iter = quality_iter.iter();
    let open_on_iter = streams
        .lock()
        .as_ref()
        .expect("Mutex lock poisoned")
        .streams_to_open_on
        .clone();
    let mut open_on_iter = open_on_iter.iter();
    let close_on_iter = streams
        .lock()
        .as_ref()
        .expect("Mutex lock poisoned")
        .streams_to_close_on
        .clone();
    let mut close_on_iter = close_on_iter.iter();
    for streamer in streamers {
        file_watcher_twitch_websocket_sender
            .send(streamer.1)
            .await
            .unwrap();
        file_watcher_event_handler_sender
            .send(StreamConfig {
                name: streamer.0.clone(),
                id: streamer.1,
                quality_overides: quality_iter.next().unwrap_or(&Vec::new()).clone(),
                streams_to_close_on: open_on_iter.next().unwrap_or(&Vec::new()).clone(),
                streams_to_open_on: close_on_iter.next().unwrap_or(&Vec::new()).clone(),
            })
            .await
            .unwrap();
    }
    // TODO: Add functionality to watch schedule file and send changes
}

#[cfg(test)]
mod test {
    use crate::watcher::StreamConfig;

    use super::*;

    #[tokio::test]
    async fn send_3_messages() {
        use tokio::sync::mpsc;

        let json_streams = std::fs::read_to_string("./tests/resources/schedule.json").unwrap();
        let streams: Streams = serde_json::from_str(&json_streams).unwrap();
        let streams = Arc::new(Mutex::new(streams));
        let path = std::path::Path::new("./tests/resources");

        let kai = StreamConfig {
            name: String::from("kaicenat"),
            id: 641_972_806,
            quality_overides: vec![(String::from("normal"), 480), (String::from("low-data"), 0)],
            streams_to_close_on: Vec::new(),
            streams_to_open_on: Vec::new(),
        };
        let hasan = StreamConfig {
            name: String::from("hasanabi"),
            id: 207_813_352,
            quality_overides: vec![(String::from("normal"), 480), (String::from("low-data"), 0)],
            streams_to_close_on: Vec::new(),
            streams_to_open_on: Vec::new(),
        };
        let jynxzi = StreamConfig {
            name: String::from("jynxzi"),
            id: 411_377_640,
            quality_overides: vec![(String::from("normal"), 480), (String::from("low-data"), 0)],
            streams_to_close_on: Vec::new(),
            streams_to_open_on: Vec::new(),
        };

        let (id_sender, mut id_reciever) = mpsc::channel(5);
        let (config_sender, mut config_reciever) = mpsc::channel(5);

        file_watcher(id_sender, config_sender, path, &streams).await;
        assert_eq!(id_reciever.recv().await, Some(641_972_806));
        assert_eq!(id_reciever.recv().await, Some(207_813_352));
        assert_eq!(id_reciever.recv().await, Some(411_377_640));
        assert_eq!(id_reciever.recv().await, None);
        assert_eq!(config_reciever.recv().await, Some(kai));
        assert_eq!(config_reciever.recv().await, Some(hasan));
        assert_eq!(config_reciever.recv().await, Some(jynxzi));
        assert_eq!(config_reciever.recv().await, None);
    }
}
