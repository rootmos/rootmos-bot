#[macro_use] extern crate lazy_static;

extern crate rootmos_bot;
use rootmos_bot::free_runner::*;
use rootmos_bot::irc::*;
use rootmos_bot::db;
use rootmos_bot::db::KV;

extern crate time;
use time::now_utc;

extern crate regex;
use regex::Regex;

extern crate sha2;
use sha2::Sha256;
use sha2::Digest;

use std::path::Path;

fn hash(s: String) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(s.as_str());
    hasher.result_str()
}

fn tag_bot<KV>(event: Event<ChatEvent>, kv: &mut KV) -> Option<Effect<ChatEffect, ()>> where KV: db::KV<String, String> {
    println!("Event: {:?}", event);
    lazy_static! {
        static ref CMD: Regex = Regex::new(r"^!(list)\s+(#[a-zA-Z0-9]+)$").unwrap();
        static ref TAGGED: Regex = Regex::new(r"(\s|^)(#[a-zA-Z0-9]+)(\s|$)").unwrap();
    }
    match event {
        Event::Event { event: ChatEvent::ChannelMsg { channel, msg, .. }, .. } =>
            if CMD.is_match(msg.as_str()) {
                match CMD.captures(msg.as_str()) {
                    Some(cap) =>
                        match (cap.at(1), cap.at(2)) {
                            (Some("list"), Some(tag)) => effect(list_cmd(channel, tag.to_owned(), kv)),
                            _ => panic!()
                        },
                    _ => panic!(),
                }
            } else if TAGGED.is_match(msg.as_str()) {
                match TAGGED.captures(msg.as_str()) {
                    Some(cap) =>
                        match cap.at(2) {
                            Some(tag) => {
                                let msg_hash = hash(msg.clone());
                                let key = format!("{}-{}-{}", channel, tag, msg_hash);
                                kv.put(&key, &msg).unwrap();
                                effect(ChatEffect::ChannelMsg { channel: channel, msg: "Tagged line".to_owned() })
                            },
                            _ => noop(),
                        },
                    None => noop(),
                }
            } else {
                noop()
            },
        _  => noop(),
    }
}

fn list_cmd<KV>(channel: String, tag: String, kv: &KV) -> ChatEffect where KV: db::KV<String, String> {
    let tag_prefix = format!("{}-{}-", channel, tag);
    println!("{}", tag_prefix);
    match kv.get_prefix(&tag_prefix).get(0) {
        Some(p) => ChatEffect::ChannelMsg { channel: channel, msg: p.1.clone() },
        _ => panic!(),
    }
}

#[test]
fn save_line_without_tag_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let input = Event::Event { time: now_utc(), event: ChatEvent::ChannelMsg {
        channel: "my_channel".to_owned(),
        from: "user1".to_owned(),
        msg: "test line tag".to_owned() } };

    assert_eq!(tag_bot(input, &mut kv), None)
}

#[test]
fn ignore_line_with_almost_tags_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let input = Event::Event { time: now_utc(), event: ChatEvent::ChannelMsg {
        channel: "my_channel".to_owned(),
        from: "user1".to_owned(),
        msg: "not#tag #not(a-tag)".to_owned() } };

    assert_eq!(tag_bot(input, &mut kv), None)
}

#[test]
fn save_line_with_tag_in_the_middle_test() {
    let tag = "#tag".to_owned();
    save_line_with_tag_test(format!("test line {} some more text", tag), tag)
}

#[test]
fn save_line_with_tag_at_the_end_test() {
    let tag = "#tag".to_owned();
    save_line_with_tag_test(format!("another test line {}", tag), tag)
}

#[test]
fn save_line_with_tag_at_the_start_test() {
    let tag = "#tag".to_owned();
    save_line_with_tag_test(format!("{} yet another test line", tag), tag)
}


#[cfg(test)]
fn save_line_with_tag_test(line: String, tag: String) {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let channel = "my_channel".to_owned();
    let input = Event::Event { time: now_utc(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user1".to_owned(),
        msg: line.clone() } };

    match tag_bot(input, &mut kv) {
        Some(Effect::Effect(ChatEffect::ChannelMsg { channel: to_channel, .. })) => {
            assert_eq!(to_channel, channel)
        },
        _ => panic!(),
    }

    let expected_key = format!("{}-{}-{}", channel, tag, hash(line.clone()));
    assert_eq!(kv.get(&expected_key).unwrap(), Some(line))
}

#[test]
fn recall_tag_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let channel = "my_channel".to_owned();
    let tag = "#tag".to_owned();
    let line = format!("a test line {}", tag);

    let input_line = Event::Event { time: now_utc(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user1".to_owned(),
        msg: line.clone() } };
    match tag_bot(input_line, &mut kv) {
        Some(_) => (),
        _ => panic!(),
    }

    let recall_cmdline = format!("!list {}", tag);
    let recall_event = Event::Event { time: now_utc(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user2".to_owned(),
        msg: recall_cmdline } };

    match tag_bot(recall_event, &mut kv) {
        Some(Effect::Effect(ChatEffect::ChannelMsg { channel: to_channel, msg })) => {
            assert_eq!(to_channel, channel);
            assert!(msg.contains(line.as_str()))
        },
        _ => panic!(),
    }
}

fn main() {
    let kv = db::rocksdb_kv::RocksDBKV::new(Path::new("tag_bot_db"));
    rootmos_bot::irc::run("irc-config.json", tag_bot, kv);
}
