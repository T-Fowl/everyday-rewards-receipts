use std::path::{Path, PathBuf};

use clap::Parser;

use crate::client::{ActivityFeedIterator, EverydayRewardsClient, EverydayRewardsError};

mod models;
mod client;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    token: String,

    #[arg(short, long, value_name = "PATH", default_value = "./receipts")]
    output: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    let client = EverydayRewardsClient::create(args.token.as_str())?;


    let path = &args.output.clone();
    std::fs::create_dir_all(path)?;


    // Remove This_Month at the start of the current run - it'll be recreated
    // On each new month all the files inside This_Month will be downloaded
    // Into LastMonth_Year
    if Path::exists(path.join("This_Month").as_path()) {
        std::fs::remove_dir_all(path.join("This_Month"))?;
    }


    let iter = ActivityFeedIterator::create(&client);
    for activity in iter {
        let activity = activity?;

        for group in activity.groups {
            let path = path.join(&group.id);
            std::fs::create_dir_all(&path)?;

            println!("{}", group.id);

            for item in group.items.iter().flatten() {
                println!("    {:?}", item);

                let path = path.join(&item.id);
                let pdf_path = path.with_extension("pdf");
                let json_path = path.with_extension("json");

                if Path::exists(&pdf_path) && Path::exists(&json_path) {
                    println!("        Skipping....");
                    continue;
                }

                let receipt = client.transaction_details(item.receipt.receipt_id.as_str())?;

                println!("        {:?}", receipt.value);

                let url = receipt.value.receipt_details.download.url;

                println!("        Downloading...");
                client.download_receipt(url.as_str(), pdf_path)?;

                std::fs::write(json_path, receipt.source)?;
            }
        }
    }


    Ok(())
}
