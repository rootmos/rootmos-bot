extern crate rootmos_bot;

use rootmos_bot::free_runner::*;
use rootmos_bot::irc::*;
use rootmos_bot::db;

use std::path::Path;

extern crate sha2;
use sha2::Sha256;
use sha2::Digest;

extern crate rand;
use rand::Rng;

fn quotes_bot<KV>(event: Event<ChatEvent>, kv: &mut KV) -> Option<Effect<ChatEffect, ()>> where KV: db::KV<String, String> {
    println!("Event: {:?}", event);
    match event {
        Event::Event { event: ChatEvent::PrivateMsg { from, msg }, .. } => {
            let mut hasher = Sha256::new();
            hasher.input_str(msg.as_str());
            let key = format!("quote-{}", hasher.result_str());
            kv.put(&key, &msg).unwrap();
            effect(ChatEffect::PrivateMsg { to: from.clone(), msg: format!("Stored quote: {}", msg)})
        },
        Event::Event { event: ChatEvent::ChannelMsg { ref channel, ref msg, .. }, .. } if msg.contains("quote") => {
            let quotes = kv.get_prefix(&"quote-".to_owned());
            match rand::thread_rng().choose(&quotes) {
                Some(p) => effect(ChatEffect::ChannelMsg { channel: channel.clone(), msg: p.1.clone()}),
                None => effect(ChatEffect::ChannelMsg { channel: channel.clone(), msg: "I don't remember any quotes...".to_owned()}),
            }
        }
        _  => noop(),
    }
}

fn main() {
    let kv = db::rocksdb_kv::RocksDBKV::new(Path::new("quotes_bot_db"));
    rootmos_bot::irc::run("irc-config.json", quotes_bot, kv);
}
