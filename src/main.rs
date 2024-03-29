mod downloader;
mod extractor;
mod grinder;
mod mapper;

use std::io::Cursor;
use std::path::PathBuf;

use anyhow::Error;
use clap::{Parser, ValueEnum};
use mapper::AtPacks2SVDsVersionMap;
use reqwest::Url;
use strum::{Display, EnumVariantNames};
use tokio;

use crate::downloader::Downloader;
use crate::extractor::extract_svds_from_pack;
use crate::grinder::Grinder;

#[derive(ValueEnum, Clone, Debug, Display, EnumVariantNames)]
#[strum(serialize_all = "SCREAMING-KEBAB-CASE")] // This is tricky for future
#[allow(non_camel_case_types)]
enum ChipsFamily {
    SAME51,
    SAME52,
    SAME53,
    SAME54,
    SAME70,
    SAMS70,
    SAMV70,
    SAMV71,
    SAMV71_RT,
    SAMRH707,
    SAMRH71,
}

/// Harvests SVDs by scrapping ATPACKs repository
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address of the repository with ATPACKs
    #[arg(short, long)]
    repository: Url,

    /// Chips family to process (eg. SAMS70)
    #[arg(short, long = "family", value_enum)]
    families: Vec<ChipsFamily>,

    /// Destination directory
    #[arg(short, long)]
    destination: Option<PathBuf>,

    /// Mapping file to update or create, contains SVD's source ATPACK version
    #[arg(short, long)]
    mapping: Option<PathBuf>,

    /// Controls verbosity levels (unsupported at the moment)
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let downloader = Downloader::new(args.repository.clone())?;
    let repository = downloader.load_repository().await?;

    println!(
        "Downloaded {} characters from the {} website .",
        repository.len(),
        args.repository
    ); // TODO: make log

    let grinder = Grinder::new(&repository);
    let collections = grinder.process_packs()?;

    let mut mappings = if let Some(ref path) = args.mapping {
        Some(if path.exists() {
            AtPacks2SVDsVersionMap::load(path)?
        } else {
            AtPacks2SVDsVersionMap::new()
        })
    } else {
        None
    };

    for collection in collections {
        print!("* Obtaining ATPACKs for {} family...", collection.family());
        if let Some(pack) = collection.packs().first() {
            if !args.families.is_empty()
                && !args
                    .families
                    .iter()
                    .any(|e| e.to_string() == collection.family())
            {
                println!(" ignoring family not requested.");
                continue;
            }

            println!(" chips found are {}", pack.chips().join(", "));

            let content = downloader.load_file(pack.archive()).await?;
            let mut reader = Cursor::new(content.as_ref());
            let svds = extract_svds_from_pack(
                &mut reader,
                args.destination.as_ref().unwrap_or(&PathBuf::from(".")),
            )?;

            println!("** Downloaded and extracted: {}", svds.join(", "));

            if let Some(ref mut m) = mappings {
                svds.iter().for_each(|s| {
                    m.add_or_update(s, pack.version());
                });
            };
        } else {
            eprintln!("** No ATPACKS for the {} family!", collection.family());
        }
    }

    if let Some(ref mut m) = mappings {
        if let Some(ref path) = args.mapping {
            // TODO: chaining unstable https://github.com/rust-lang/rust/issues/53667
            m.save(path)?;
        }
    }

    Ok(())
}
