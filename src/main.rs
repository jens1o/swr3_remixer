extern crate dotenv;
extern crate json;
extern crate reqwest;

mod swr3_api;
mod youtube_api;

use dotenv::dotenv;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    dotenv().ok();

    let sleep_duration = Duration::from_secs(30);
    let mut playing_song: Option<swr3_api::Swr3Song> = None;

    // TODO: Refactor errors to be fatal (in certain circumstances)

    loop {
        if let Some(song_data) = swr3_api::get_current_played_song() {
            if playing_song != Some(song_data.clone()) {
                println!("Searching for remix of {} …", &song_data);
                if let Some(video_url) =
                    youtube_api::get_video_search_result_url(get_yt_search_query(song_data.clone()))
                {
                    enqueue_vlc_playlist(
                        download_video(video_url).expect("Cannot download video!"),
                    );
                    playing_song = Some(song_data);
                }
            }
        }

        sleep(sleep_duration);
    }
}

fn download_video(url: String) -> Option<String> {
    println!("Downloading {} …", url);
    let ytdl_args = [
        "-x",
        "--no-playlist",
        "--ignore-config",
        "--print-json",
        "-o",
        "downloads/%(id)s.%(acodec)s",
        &url,
    ];

    let youtube_dl = Command::new("youtube-dl")
        .args(&ytdl_args)
        .output()
        .expect("Cannot get output of finished child process!");

    let buffer =
        String::from_utf8(youtube_dl.stdout).expect("Invalid encoding in youtube_dl output!");

    let json = json::parse(&buffer).expect("youtube_dl returned no valid json!");

    Some(format!("downloads/{}.{}", json["id"], json["acodec"]))
}

fn get_yt_search_query(song: swr3_api::Swr3Song) -> String {
    format!("{} {} remix", song.artist, song.title)
}

fn enqueue_vlc_playlist(uri: String) {
    println!("Playing {} with vlc!", &uri);

    let mut command = Command::new("vlc");

    command
        .args(&["--started-from-file", "--playlist-enqueue"])
        .arg(uri);

    command.spawn().unwrap();
}
