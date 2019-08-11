use serde::Deserialize;
use std::fmt;

const API_URL: &'static str = "https://www.swr3.de/ext/cf=42/actions/feed/onair.json";

#[derive(Debug, Deserialize)]
pub struct Swr3ApiResponse {
    pub playlist: Vec<Swr3Song>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Swr3Song {
    pub title: String,
    pub artist: String,
}

impl fmt::Display for Swr3Song {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#""{}" by {}"#, self.title, self.artist)
    }
}

pub fn get_current_played_song() -> Option<Swr3Song> {
    let response = reqwest::get(API_URL);

    response
        .and_then(|mut result| result.json::<Swr3ApiResponse>())
        .ok()
        .map(|x| x.playlist)
        .and_then(|x| x.iter().nth(0).cloned())
}
