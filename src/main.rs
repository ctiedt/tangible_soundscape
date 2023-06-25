use rodio::{decoder::LoopedDecoder, Decoder, OutputStream, Sink, Source};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{sink, BufRead, BufReader},
    sync::mpsc::Receiver,
    time::Duration,
};

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
struct MiniInfo {
    category: String,
    kind: String,
    id: u32,
}

struct MiniSound {
    decoder: LoopedDecoder<File>,
}

impl From<&MiniInfo> for MiniSound {
    fn from(value: &MiniInfo) -> Self {
        let fp = format!(
            "{}/sounds/{}/{}.wav",
            env!("CARGO_MANIFEST_DIR"),
            value.category,
            value.kind
        );
        let file = std::fs::File::open(fp).unwrap();
        let decoder = Decoder::new_looped(file).unwrap();

        Self { decoder }
    }
}

impl Iterator for MiniSound {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next()
    }
}

impl Source for MiniSound {
    fn current_frame_len(&self) -> Option<usize> {
        self.decoder.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.decoder.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.decoder.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.decoder.total_duration()
    }
}

fn sound_thread(rx: Receiver<HashSet<MiniInfo>>) {
    let previous_state = rx.recv().unwrap();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sinks = HashMap::new();
    while let Ok(state) = rx.recv() {
        //if state != previous_state {}
        let ids = state.iter().map(|it| it.id).collect::<Vec<_>>();
        for info in state {
            if sinks.get(&info.id).is_none() {
                let sink = Sink::try_new(&stream_handle).unwrap();
                sink.append(MiniSound::from(&info));
                sinks.insert(info.id, sink);
            }
        }

        for (id, sink) in sinks.iter() {
            if !ids.contains(id) {
                sink.stop();
            }
        }
        sinks.retain(|id, _| ids.contains(id));
    }
}

fn main() -> color_eyre::Result<()> {
    let port = serialport::new("COM3", 9600)
        .timeout(Duration::from_millis(1000))
        .open()?;
    let mut reader = BufReader::new(port);

    let mut buf = String::new();

    let mut reading = false;

    let mut state = HashSet::new();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(|| sound_thread(rx));

    loop {
        reader.read_line(&mut buf)?;
        let line = buf.trim().to_owned();
        buf.clear();
        if line == "===START===" {
            state.clear();
            reading = true;
            continue;
        }
        if line == "===END===" {
            reading = false;
            tx.send(state.clone())?;
            continue;
        }
        if reading && !line.is_empty() {
            if let Ok(info) = serde_json::from_str::<MiniInfo>(&line) {
                //println!("{:?}", info);
                state.insert(info);
            }
        }
    }
}
