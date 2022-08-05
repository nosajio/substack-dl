use reqwest;
use rss::Channel;
use std::error::Error;

// interface:
// substack_export name.substack.com

#[derive(Debug)]
struct Post {
    slug: String,
    title: String,
    body: String,
    pubdate: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = std::env::args().nth(1).expect("no url provided");
    let rss_url = get_substack_url(url).expect("error resolving substack url");
    let feed = fetch_feed(rss_url).await?;
    let posts = parse_posts(feed);
    for p in posts.unwrap() {
        println!("{:?}\n\n", p);
    }

    Ok(())
}

fn parse_posts(chan: Channel) -> Result<Vec<Post>, Box<dyn Error>> {
    let mut posts = Vec::new();
    for item in chan.items() {
        posts.push(Post {
            slug: String::from(item.link().unwrap()),
            title: String::from(item.title().unwrap()),
            pubdate: String::from(item.pub_date().unwrap()),
            body: String::from(item.content().unwrap()),
        })
    }
    return Ok(posts);
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

async fn fetch_feed(rss_url: String) -> Result<Channel, Box<dyn Error>> {
    println!("request url: {}", rss_url);
    let res = reqwest::get(rss_url).await?.bytes().await?;
    let channel = Channel::read_from(&res[..])?;
    Ok(channel)
}
