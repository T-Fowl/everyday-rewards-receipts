use std::fmt::Debug;
use std::path::{Path, PathBuf};

use clap::Parser;

use crate::client::{ActivityFeedIterator, EverydayRewardsClient, EverydayRewardsError, ValueWithSource};
use crate::models::ReceiptDetailsResponse;

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
    if Path::exists(path.join("Last_Month").as_path()) {
        std::fs::remove_dir_all(path.join("Last_Month"))?;
    }


    let iter = ActivityFeedIterator::create(&client);
    for activity in iter {
        if let Ok(activity) = activity {
            for group in activity.groups {
                let path = path.join(&group.id);

                match std::fs::create_dir_all(&path) {
                    Err(e) => {
                        eprintln!("Could not create directory for group {}. Skipping", group.id);
                    }
                    Ok(_) => {
                        println!("{}", group.id);

                        for item in group.items.iter().flatten() {
                            println!("    {:?}", item);

                            if let Some(item_receipt) = &item.receipt {
                                match client.transaction_details(item_receipt.receipt_id.as_str()) {
                                    Err(e) => {
                                        eprintln!("Error when fetching transaction details for {}. Skipping", &item.id);
                                    }
                                    Ok(receipt) => {
                                        println!("        {:?}", receipt.value);


                                        let pdf_path = path.join(&receipt.value.receipt_details.download.filename);
                                        let json_path = pdf_path.with_extension("json");

                                        if Path::exists(&pdf_path) && Path::exists(&json_path) {
                                            println!("        Skipping as a pdf and json file already exist...");
                                            continue;
                                        }

                                        let url = receipt.value.receipt_details.download.url;

                                        println!("        Downloading...");

                                        if let Err(e) = client.download_receipt(url.as_str(), pdf_path) {
                                            eprintln!("An error occurred downloading the receipt: {}", e)
                                        } else {
                                            if let Err(e) = std::fs::write(json_path, receipt.source) {
                                                eprintln!("An error occurred writing the receipt json to disk: {}", e)
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!("Skipping {} as there is no associated receipt.", item.id);
                            }
                        }
                    }
                }
            }
        }
    }


    Ok(())
}
