use serde::Deserialize;
use std::env;

const YOUTUBE_SEARCH_API_BASE_URL: &'static str =
    "https://www.googleapis.com/youtube/v3/search?part=snippet";

#[derive(Debug, Deserialize)]
struct YoutubeApiSearchResponse {
    items: Vec<YouTubeApiSearchResult>,
}

#[derive(Debug, Deserialize)]
struct YouTubeApiSearchResult {
    id: YoutubeVideoId,
}

#[derive(Debug, Deserialize)]
struct YoutubeVideoId {
    #[serde(rename = "videoId")]
    video_id: String,
}

pub fn get_video_search_result_url(query: String) -> Option<String> {
    let client = reqwest::Client::new();

    // TODO: Add caching logic

    let response = client
        .get(YOUTUBE_SEARCH_API_BASE_URL)
        .query(&[("key", env::var("YT_API_KEY").expect("No YouTube API key given. Please ensure there is a .env file with the necessary key registered at Google"))])
        .query(&[("q", query)])
        .send();

    response
        .and_then(|mut response| response.json::<YoutubeApiSearchResponse>())
        .ok()
        .map(|x| x.items[0].id.video_id.clone())
        .map(get_video_url)
}

fn get_video_url(video_id: String) -> String {
    format!("https://www.youtube.com/watch?v={}", video_id)
}
