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
    let mut h = hasher.result_str();
    h.truncate(8);
    h
}

fn tag_bot<KV>(event: Event<ChatEvent>, kv: &mut KV) -> Option<Effect<ChatEffect, ()>> where KV: db::KV<String, String> {
    println!("Event: {:?}", event);
    lazy_static! {
        static ref LIST_CMD: Regex = Regex::new(r"^!list\s+(#[a-zA-Z0-9]+)$").unwrap();
        static ref TAGGED: Regex = Regex::new(r"(\s|^)(#[a-zA-Z0-9]+)(\s|$)").unwrap();
    }

    match event {
        Event::Event { event: ChatEvent::ChannelMsg { channel, msg, .. }, .. } =>
            if let Some(cap) = LIST_CMD.captures(msg.as_str()) {
                let tag = cap.at(1).unwrap();
                effect(list_cmd(channel, tag.to_owned(), kv))
            } else if let Some(cap) = TAGGED.captures(msg.as_str()) {
                let tag = cap.at(2).unwrap();
                effect(tag_line(channel, tag.to_owned(), msg.clone(), kv))
            } else {
                noop()
            },
        _  => noop(),
    }
}

fn list_cmd<KV>(channel: String, tag: String, kv: &KV) -> ChatEffect where KV: db::KV<String, String> {
    let tag_prefix = format!("{}-{}-", channel, tag);
    println!("{}", tag_prefix);
    let mut output = format!("Listing tag: {}", tag);
    for p in kv.get_prefix(&tag_prefix) {
        output.push_str(p.1.as_str());
        output.push('\n')

    }
    ChatEffect::ChannelMsg { channel: channel, msg: output }
}

fn tag_line<KV>(channel: String, tag: String, line: String, kv: &mut KV) -> ChatEffect where KV: db::KV<String, String> {
    let line_hash = hash(line.clone());
    let key = format!("{}-{}-{}", channel, tag, line_hash);
    kv.put(&key, &line).unwrap();
    let response = format!("Line tagged, recall using: !list {}", tag);
    ChatEffect::ChannelMsg { channel: channel, msg: response }
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
fn recall_tag_one_line_test() {
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

#[test]
fn recall_tag_several_lines_test() {
    let mut kv = db::hashmap_kv::HashMapKV::new();
    let channel = "my_channel".to_owned();
    let tag = "#tag".to_owned();
    let line1 = format!("a test line {}", tag);
    let line2 = format!("another {} test line", tag);
    let line3 = format!("{} yet another line", tag);

    run_tag_bot_for_line_in_channel(&channel, &"user1".to_owned(), &line1, &mut kv);
    run_tag_bot_for_line_in_channel(&channel, &"user2".to_owned(), &line2, &mut kv);
    run_tag_bot_for_line_in_channel(&channel, &"user3".to_owned(), &line3, &mut kv);

    let recall_cmdline = format!("!list {}", tag);
    let recall_event = Event::Event { time: now_utc(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: "user4".to_owned(),
        msg: recall_cmdline } };

    match tag_bot(recall_event, &mut kv) {
        Some(Effect::Effect(ChatEffect::ChannelMsg { channel: to_channel, msg })) => {
            assert_eq!(to_channel, channel);
            assert!(msg.contains(line1.as_str()));
            assert!(msg.contains(line2.as_str()));
            assert!(msg.contains(line3.as_str()))
        },
        _ => panic!(),
    }
}

#[cfg(test)]
fn run_tag_bot_for_line_in_channel<KV>(channel: &String, user: &String, line: &String, kv: &mut KV) -> () where KV: db::KV<String, String> {
    let input = Event::Event { time: now_utc(), event: ChatEvent::ChannelMsg {
        channel: channel.clone(),
        from: user.clone(),
        msg: line.clone() } };
    match tag_bot(input, kv) {
        Some(_) => (),
        _ => panic!(),
    }
}


fn main() {
    let kv = db::rocksdb_kv::RocksDBKV::new(Path::new("tag_bot_db"));
    rootmos_bot::irc::run("irc-config.json", tag_bot, kv);
}
