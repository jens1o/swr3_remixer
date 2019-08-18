use serde::Deserialize;
use std::env;

const YOUTUBE_SEARCH_API_BASE_URL: &'static str =
    "https://www.googleapis.com/youtube/v3/search?part=snippet&maxResults=1";

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

cached! {
    VID_MAPPING;
    fn get_video_search_result_url(query: String) -> Option<String> = {
        let client = reqwest::Client::new();

        // TODO: Add caching logic

        let response = client
            .get(YOUTUBE_SEARCH_API_BASE_URL)
            .query(&[("key", env::var("YT_API_KEY").expect("No YouTube API key given. Please ensure there is a .env file with the necessary key registered at Google"))])
            .query(&[("q", query)])
            .send();

        let result = response
            .and_then(|mut response| response.json::<YoutubeApiSearchResponse>());

        if let Err(err) = result {
            eprintln!("Error while parsing response: {}", err);
            return None;
        }

        return Some(get_video_url(result.unwrap().items[0].id.video_id.clone()));
    }
}

fn get_video_url(video_id: String) -> String {
    format!("https://www.youtube.com/watch?v={}", video_id)
}
