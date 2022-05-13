use std::{io::BufRead, process::Command, path::Path};

use lazy_static::lazy_static;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use regex::Regex;

#[derive(Debug, Clone)]
struct VideoInfo {
    link: String,
    title: Option<String>,
}

fn extract_playlist_links(playlist_link: &str) -> Vec<String> {
    let output = Command::new("yt-dlp")
        .arg("--flat-playlist")
        .args(["--print", "url"])
        .arg(playlist_link)
        .output()
        .unwrap();

    output
        .stdout
        .lines()
        .map(|l| l.unwrap())
        .collect::<Vec<String>>()
}

fn extract_title(link: &str) -> String {
    let output = Command::new("yt-dlp")
        .arg("--print")
        .arg("title")
        .arg("--skip-download")
        .arg(link)
        .output()
        .unwrap();

    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
    lazy_static! {
        static ref TITLE_REGEX: Regex = Regex::new(r"^[^||\[|\(|]*").unwrap();
    }
    let mat = TITLE_REGEX.find(title.as_str()).unwrap();
    let title = title[mat.start()..mat.end()].trim().to_string();
    title
}

fn download_video(video: &VideoInfo) {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-x")
        .args(["--audio-format", "mp3"])
        .arg(&video.link);

    if video.title.is_some() {
        let mut title = String::from("./audio/");
        title.push_str(video.title.as_ref().unwrap().as_str());
        title.push_str(".%(ext)s'");
        cmd.args(["-o", &title]);
    }
    cmd.output().unwrap();
}

fn main() {
    if !Path::new("./audio/").exists() {
        std::fs::create_dir("./audio").unwrap();  
    }

    if !Path::new("./to_download.txt").exists() {
        std::fs::write("./to_download.txt", "").unwrap();
    }


    println!("Reading to_download.txt");
    let input = std::fs::read_to_string("./to_download.txt").unwrap();
    let raw_input_links: Vec<String> = input.lines().map(|l| l.to_string()).collect();

    let playlist_links: Vec<&String> = raw_input_links
        .iter()
        .filter(|x| x.contains("&list"))
        .collect();

    let raw_video_links: Vec<&String> = raw_input_links
        .iter()
        .filter(|x| !x.contains("&list"))
        .collect();

    let mut videos: Vec<VideoInfo> = Vec::new();

    for link in raw_video_links {
        videos.push(VideoInfo {
            link: link.to_owned(),
            title: None,
        });
    }

    println!("Extracting playlist video links");

    let playlist_video_links = playlist_links
        .par_iter()
        .map(|link| extract_playlist_links(link))
        .flatten()
        .collect::<Vec<String>>();
    for link in playlist_video_links {
        videos.push(VideoInfo {
            link: link.to_owned(),
            title: None,
        });
    }

    println!("Getting video titles and adjusting them");

    videos
        .par_iter_mut()
        .for_each(|v| v.title = Some(extract_title(&v.link)));

    println!("Downloading videos");

    videos.par_iter().for_each(download_video);

    println!("Done.");
}
