use html2md::parse_html;
use rss::Channel;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
// use std::path::Path;

pub struct Parser {
    pub output_dir: String,
    pub url: String,
    pub items: Vec<Post>,
}

#[derive(Debug)]
pub enum ParserError {
    NoItems,
    ParseHTML,
    FetchItems,
    SaveItems,
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
                html,
                md.as_str(),
                // [TODO] Split the slug from the URL
                item.link().unwrap(),
                item.title().unwrap(),
                item.pub_date().unwrap(),
            ));
        }
        self.items = items;
        Ok(())
    }

    pub fn save_files(self) -> Result<(), ParserError> {
        println!("Saving files in {}", self.output_dir);
        for it in self.items.iter() {
            println!("Write... {}", it.slug);
            write_file(self.output_dir.as_str(), it)?;
        }
        Ok(())
    }
}

fn write_file(dir: &str, item: &Post) -> Result<(), ParserError> {
    // First save into tmp, and move to provided directory
    // [todo] actually use /tmp, not the random dir this returns??
    let tmp = env::temp_dir();
    let file_full_path = tmp.join(dir).join(item.filename());
    // [todo] check if this dir exists and make it
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
    slug: String,
    title: String,
    md: String,
    html: String,
    pubdate: String,
}

impl Post {
    fn new(html: &str, md: &str, slug: &str, title: &str, pubdate: &str) -> Self {
        let post = Post {
            slug: String::from(slug),
            title: String::from(title),
            pubdate: String::from(pubdate),
            md: String::from(md),
            html: String::from(html),
        };
        post
    }

    fn filename(&self) -> String {
        format!("{}.md", "test")
    }
}
