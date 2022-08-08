use chrono::{DateTime, FixedOffset};
use core::fmt;
use html2md::parse_html;
use rss::Channel;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Parser {
    pub output_dir: String,
    pub url: String,
    pub items: Vec<Post>,
}

#[derive(Debug)]
pub enum ParserError {
    SaveItems,
    NoOverwrite,
    CantDelete,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ParserError::CantDelete => "Unable to delete directory",
            ParserError::NoOverwrite => "Overwrite not allowed",
            ParserError::SaveItems => "Successfully saved all items",
        };
        write!(f, "{}", s)
    }
}

pub enum SaveStatus {
    Success,
}

impl fmt::Display for SaveStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SaveStatus::Success => "Successfully saved posts",
        };
        write!(f, "{}", s)
    }
}

impl Parser {
    pub fn new(url: String, output_dir: String) -> Self {
        let rss_url = get_substack_url(url).expect("error resolving substack url");
        Parser {
            url: rss_url,
            output_dir,
            items: Vec::new(),
        }
    }

    pub async fn fetch_and_parse(&mut self) -> Result<(), Box<dyn Error>> {
        let feed = fetch_feed(self.url.to_owned()).await?;
        let mut items = Vec::new();
        for item in feed.items() {
            let html = item.content().unwrap();
            let md = parse_html(&html);
            items.push(Post::new(
                md.as_str(),
                item.link().unwrap(),
                item.title().unwrap(),
                item.pub_date().unwrap(),
            ));
        }
        self.items = items;
        Ok(())
    }

    pub fn save_dir_exists(&self) -> bool {
        let dir = get_save_dir(&self.output_dir);
        does_dir_exist(&dir)
    }

    pub fn save_files(self, overwrite: bool) -> Result<SaveStatus, ParserError> {
        println!("Saving files in {}", self.output_dir);
        let save_dir = get_save_dir(&self.output_dir);

        match does_dir_exist(&save_dir) {
            true => {
                if !overwrite {
                    return Err(ParserError::NoOverwrite);
                }
                delete_dir(&save_dir)?;
            }
            false => {
                fs::create_dir_all(&save_dir).unwrap();
            }
        }

        for it in self.items.iter() {
            println!("Write... {}", it.slug().unwrap());
            write_file(self.output_dir.as_str(), it)?;
        }
        Ok(SaveStatus::Success)
    }
}

fn get_save_dir(dir: &str) -> PathBuf {
    let tmp = Path::new("/tmp");
    let dir_path = tmp.join(dir);
    dir_path
}

fn delete_dir(dir: &PathBuf) -> Result<bool, ParserError> {
    match fs::remove_dir_all(dir) {
        Ok(_) => Ok(true),
        Err(_) => Err(ParserError::CantDelete),
    }
}

fn does_dir_exist(dir: &PathBuf) -> bool {
    match fs::metadata(dir) {
        Ok(md) => md.is_dir(),
        Err(_) => false,
    }
}

fn write_file(dir: &str, item: &Post) -> Result<(), ParserError> {
    // First save into tmp, and move to provided directory
    let tmp = Path::new("/tmp");
    let dir_path = tmp.join(dir);
    let file_full_path = dir_path.join(item.filename());

    if !match fs::metadata(&dir_path) {
        Ok(md) => md.is_dir(),
        Err(_) => false,
    } {
        fs::create_dir_all(&dir_path).unwrap();
    }

    println!(
        "Write file: {}, {}",
        item.title,
        String::from(file_full_path.to_str().unwrap())
    );
    let mut f = match File::create(&file_full_path) {
        Ok(file) => file,
        Err(_e) => return Err(ParserError::SaveItems),
    };
    let md_bytes = item.md.as_bytes();
    match f.write_all(&md_bytes) {
        Ok(_) => Ok(()),
        Err(_e) => return Err(ParserError::SaveItems),
    }
}

async fn fetch_feed(rss_url: String) -> Result<Channel, Box<dyn Error>> {
    let res = reqwest::get(rss_url).await?.bytes().await?;
    let channel = Channel::read_from(&res[..])?;
    Ok(channel)
}

fn get_substack_url(domain: String) -> Result<String, Box<dyn Error>> {
    let mut clean_domain = domain.as_str();
    if domain.starts_with("http") {
        let d: Vec<&str> = domain.split("://").collect();
        clean_domain = d[1];
    }
    let full_url = format!("https://{}/feed", clean_domain);
    Ok(full_url)
}

pub struct Post {
    url: String,
    title: String,
    md: String,
    pubdate: DateTime<FixedOffset>,
}

impl Post {
    fn new(md: &str, url: &str, title: &str, pubdate: &str) -> Self {
        let pubdate = match DateTime::parse_from_rfc2822(pubdate) {
            Ok(d) => d,
            Err(err) => panic!("{}", err),
        };
        let post = Post {
            pubdate,
            url: String::from(url),
            title: String::from(title),
            md: String::from(md),
        };
        post
    }

    fn slug(&self) -> Result<&str, String> {
        let parts = self.url.split("/");
        let slug = match parts.last() {
            Some(slug) => Ok(slug),
            None => Err(String::from("unable to split url to find the slug")),
        };
        slug
    }

    fn filename(&self) -> String {
        let slug = self.slug().expect("unable to get slug for filename");
        let date = self.pubdate.format("%m-%d-%Y");
        format!("{}-{}.md", date, slug)
    }
}
