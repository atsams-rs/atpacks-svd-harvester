mod downloader;
mod extractor;
mod grinder;

use anyhow::Error;
use clap::{Parser, ValueEnum};
use reqwest::Url;
use tokio;
use strum::{Display, EnumVariantNames}; 

use crate::downloader::Downloader;
use crate::grinder::Grinder;

#[derive(ValueEnum, Clone, Debug, Display, EnumVariantNames)]
enum ChipsFamily {
    SAME70,
    SAMS70,
    SAMV70,
    SAMV71,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address of the repository with ATPACKs
    #[arg(short, long)]
    repository: Url,

    /// Chips family to process (eg. SAMS70)
    #[arg(short, long, value_enum)]
    family: Option<ChipsFamily>,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
   
    let downloader = Downloader::new(args.repository.clone())?;
    let repository = downloader.load_repository().await?;

    println!("Downloaded {} characters from the {} website .", repository.len(), args.repository); // TODO: make log

    let grinder = Grinder::new(&repository);
    let _collection = grinder.process_packs();

    Ok(())
}