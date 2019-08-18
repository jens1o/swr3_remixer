extern crate dotenv;
extern crate json;
extern crate reqwest;
#[macro_use]
extern crate cached;

mod swr3_api;
mod youtube_api;

use dotenv::dotenv;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    dotenv().ok();

    let mut playing_song: Option<swr3_api::Swr3Song> = None;
    let mut same_song_counter: u8 = 0;
    let mut sleep_duration = get_wait_duration(same_song_counter);

    // TODO: Refactor errors to be fatal (in certain circumstances)

    loop {
        if let Some(song_data) = swr3_api::get_current_played_song() {
            if playing_song != Some(song_data.clone()) {
                println!("Searching for a remix of {} …", &song_data);
                if let Some(video_url) =
                    youtube_api::get_video_search_result_url(get_yt_search_query(song_data.clone()))
                {
                    enqueue_vlc_playlist(
                        download_video(video_url).expect("Cannot download video!"),
                    );
                    playing_song = Some(song_data);
                    same_song_counter = 0;
                    sleep_duration = get_wait_duration(same_song_counter);
                }
            } else {
                same_song_counter = same_song_counter.wrapping_add(1);
                sleep_duration = get_wait_duration(same_song_counter);
                println!(
                    "[{}] Still playing the same song, skipping this iteration. Polling again in {:?} …",
                    same_song_counter, sleep_duration
                );
            }
        }

        sleep(sleep_duration);
    }
}

/// Returns the duration the program should wait between two tries with the same data.
/// Gets more aggressive the higher the given `try_count` is.
fn get_wait_duration(try_count: u8) -> Duration {
    Duration::from_secs(match try_count {
        0..=1 => 60,
        2..=4 => 30,
        _ => 20,
    })
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

    println!(
        r#"Downloaded the song "{}", queuing up for playing …"#,
        json["title"]
    );

    Some(format!("downloads/{}.{}", json["id"], json["acodec"]))
}

fn get_yt_search_query(song: swr3_api::Swr3Song) -> String {
    format!(
        "{} {} remix",
        song.artist
            .replace(';', "") // removing the semicolon SWR3 is using for separation of the artists increases the quality of the search results
            .replace("&#39;", "'"), // TODO: Decode html entities properly, rust support in terms with serde is pretty limited here
        song.title
    )
}

fn enqueue_vlc_playlist(uri: String) {
    println!("Playing {} with vlc!", &uri);

    let mut command = Command::new("vlc");

    command
        .args(&["--started-from-file", "--playlist-enqueue"])
        .arg(uri);

    command.spawn().unwrap();
}
