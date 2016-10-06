
#[derive(Serialize, Deserialize, Debug)]
struct TaggedLine {
    channel: String,
    tag: String,
    user: String,
    time: chrono::DateTime<UTC>,
    line: String,
    hash: String,
}
