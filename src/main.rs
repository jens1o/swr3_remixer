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

/// Maximum time to poll until we eventually give up because we don't get something new.
/// Most likely some kind of refusal/server out of business.
const MAX_WAIT_COUNT: u8 = 20;

fn main() {
    // load .env file, containing the YouTube API key
    dotenv().ok();

    // Keeps a record of the latest song that was enqueued/is playing
    let mut playing_song: Option<swr3_api::Swr3Song> = None;
    // Remembers how many times SWR3 reported that they still would play the same song.
    // This doesn't grow linear over time.
    let mut same_song_counter: u8 = 0;
    // Holds how long we're waiting in relation to how many times we already
    // have been reported back that SWR3 would play the same song.
    let mut sleep_duration = get_wait_duration(same_song_counter);

    // TODO: Refactor errors to be fatal (in certain circumstances)

    loop {
        if let Some(song_data) = swr3_api::get_current_played_song() {
            // TODO: Can we get rid of this clone here?
            if playing_song != Some(song_data.clone()) {
                println!("Searching for a remix of {} …", &song_data);
                // TODO: Can we get rid of the clone here?
                if let Some(video_url) =
                    youtube_api::get_video_search_result_url(get_yt_search_query(song_data.clone()))
                {
                    enqueue_vlc_playlist(
                        download_video(video_url).expect("Cannot download video!"),
                    );

                    // remember the new song and reset counters
                    playing_song = Some(song_data);
                    same_song_counter = 0;
                    sleep_duration = get_wait_duration(same_song_counter);
                }
            } else {
                same_song_counter += 1;

                if same_song_counter > MAX_WAIT_COUNT {
                    eprintln!(
                        "Still no new song after waiting {} times, aborting!",
                        MAX_WAIT_COUNT
                    );
                    break;
                }

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
        "downloads/%(id)s.%(acodec)s", // save the file in e.g. `Q4orjhvnkIk.opus`,
        // where `Q4orjhvnkIk` is the video id and `opus` is the audio codec of it
        &url,
    ];

    let mut youtube_dl = Command::new(if cfg!(windows) {
        "cmd.exe"
    } else {
        "youtube-dl"
    });

    if cfg!(windows) {
        youtube_dl.args(&["/C", "youtube-dl.exe"]);
    }

    let youtube_dl = youtube_dl
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

/// Returns the search query for a given `song`. The returned
/// string should be - when fed into the YouTube search - leading to
/// the best possible remix for a given song.
fn get_yt_search_query(song: swr3_api::Swr3Song) -> String {
    format!(
        r#"{} "{}" remix"#, // make sure the song title is in the search results
        song.artist
            .replace(';', "") // removing the semicolon SWR3 is using for separation of the artists increases the quality of the search results
            .replace("&#39;", "'"), // TODO: Decode html entities properly, rust support in terms with serde is pretty limited here
        song.title
    )
}

fn enqueue_vlc_playlist(uri: String) {
    println!("Enqueuing {} in vlc!", &uri);

    let mut command = Command::new(if cfg!(windows) { "cmd.exe" } else { "vlc" });

    if cfg!(windows) {
        command.args(&["/C", "vlc.exe"]);
    }

    command
        .args(&["--started-from-file", "--playlist-enqueue"])
        .arg(uri);

    command.spawn().unwrap();
}
