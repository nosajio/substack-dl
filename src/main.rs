mod parser;

use promptly::prompt_default;

// interface:
// substack-dl name.substack.com

#[tokio::main]
async fn main() -> Result<(), String> {
    let url = std::env::args().nth(1).expect("no url provided");
    let output_dir = std::env::args().nth(2).expect("no dir provided");

    let mut op = parser::Parser::new(url, output_dir);
    op.fetch_and_parse()
        .await
        .expect("Could not fetch & parse posts");

    let save_op = match op.save_dir_exists() {
        true => {
            let overwrite = prompt_default(
                format!(
                    "directory {} exists. Do you want to overwrite it?",
                    &op.output_dir
                ),
                false,
            )
            .unwrap();
            op.save_files(overwrite)
        }
        false => op.save_files(false),
    };

    match save_op {
        Ok(status) => println!("Completed: {}", status),
        Err(e) => return Err(format!("{}", e)),
    }

    Ok(())
}
