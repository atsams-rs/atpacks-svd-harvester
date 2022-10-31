mod downloader;
mod extractor;
mod grinder;

use std::io::Cursor;
use std::path::PathBuf;

use anyhow::Error;
use clap::{Parser, ValueEnum};
use reqwest::Url;
use tokio;
use strum::{Display, EnumVariantNames}; 

use crate::downloader::Downloader;
use crate::extractor::extract_svds_from_pack;
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

    /// Destination directory
    #[arg(short, long)]
    destination: Option<PathBuf>,

    /// Controls verbosity levels (unsupported at the moment)
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
    let collections = grinder.process_packs()?;

    for collection in collections {
        println!("* Obtaining ATPACKs for {} family...", collection.family());
        if let Some(pack) = collection.packs().first() {
            // TODO: collect ATPACK version into a map file (json?)

            println!("** Obtaining ATPACKS for {}", pack.chips().join(", "));

            let content = downloader.load_file(pack.archive()).await?;
            let mut reader = Cursor::new(content.as_ref());
            let svds = extract_svds_from_pack(&mut reader, args.destination.as_ref().unwrap_or(&PathBuf::from(".")))?;

            println!("** Downloaded and extracted: {}", svds.join(", "));
        } else {
            eprintln!("** No ATPACKS for the {} family!", collection.family());
        }
    }

    Ok(())
}