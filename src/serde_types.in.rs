#[derive(Serialize, Deserialize, Debug)]
struct TaggedLine {
    time_rfc3339: String,
    user: String,
    line: String,
}
