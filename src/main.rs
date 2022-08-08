mod parser;
use std::error::Error;

// interface:
// substack_export name.substack.com

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = std::env::args().nth(1).expect("no url provided");
    let output_dir = std::env::args().nth(2).expect("no dir provided");

    let mut op = parser::Parser::new(url, output_dir);
    op.fetch_and_parse()
        .await
        .expect("Could not fetch & parse posts");
    op.save_files().expect("Could not save files");

    Ok(())
}
