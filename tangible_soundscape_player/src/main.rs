use rodio::{decoder::LoopedDecoder, Decoder, OutputStream, Sink};

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    sync::mpsc::Receiver,
    time::Duration,
};
use tangible_soundscape_common::{rule::Rule, FigureInfo};

fn decoder_for_rule(rule: &Rule) -> LoopedDecoder<File> {
    let file = File::open(&rule.sound).unwrap();
    Decoder::new_looped(file).unwrap()
}

fn sound_thread(rx: Receiver<HashSet<FigureInfo>>, rules: Vec<Rule>) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sinks = HashMap::new();

    while let Ok(state) = rx.recv() {
        let figures = state.into_iter().collect::<Vec<_>>();
        for (idx, rule) in rules.iter().enumerate() {
            if rule.matches(&figures) && !sinks.contains_key(&idx) {
                let sink = Sink::try_new(&stream_handle).unwrap();
                sink.append(decoder_for_rule(rule));
                sinks.insert(idx, sink);
            }
            if !rule.matches(&figures) && sinks.contains_key(&idx) {
                let sink = sinks.remove(&idx).unwrap();
                sink.stop();
            }
        }
    }
}

fn main() -> color_eyre::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let port_name = &args[1];
    let port = serialport::new(port_name, 9600)
        .timeout(Duration::from_millis(1000))
        .open()?;
    let mut reader = BufReader::new(port);

    let rules_path = &args[2];
    let rules: Vec<Rule> =
        serde_json::from_reader(std::fs::File::open(rules_path).unwrap()).unwrap();

    let mut buf = String::new();

    let mut reading = false;

    let mut state = HashSet::new();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(|| sound_thread(rx, rules));

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
            if let Ok(info) = serde_json::from_str::<FigureInfo>(&line) {
                state.insert(info);
            }
        }
    }
}
