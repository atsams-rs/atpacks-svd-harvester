use std::{rc::Rc};

use anyhow::Error;
use lazy_regex::{regex_captures};
use semver::Version;
use strum::VariantNames;
use std::clone::Clone;
use scraper::{Html, Selector};

pub struct Grinder {
    document: Html,
}

pub struct AtPacksCollection {
    family: Rc<String>,
    chips: Rc<Vec<String>>,
    packs: Vec<AtPack>
}

impl AtPacksCollection {
    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn chips(&self) -> &[String] {
        self.chips.as_slice()
    }

    pub fn packs(&self) -> &[AtPack] {
        &self.packs
    }
}

pub struct AtPack {
    family: Rc<String>,
    version: Version,
    chips: Rc<Vec<String>>,
    archive: String,
}

impl AtPack {
    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn chips(&self) -> &[String] {
        &self.chips
    }

    pub fn archive(&self) -> &str {
        &self.archive
    }
}

impl Grinder {
    pub fn new(input: &str) -> Grinder {
        Grinder {
            document: Html::parse_document(input),
        }
    }
    
    pub fn process_packs(&self) -> Result<Vec<AtPacksCollection>, Error> {
        let panel_selector = Selector::parse("div.panel-group>div.panel").unwrap();
        let title_selector = Selector::parse("div.panel-heading>h3.panel-title>a").unwrap();
        let device_list_selector = Selector::parse("div.panel-body>div.device-list>ul.list-inline>li.device-list-item").unwrap();
        let release_selector = Selector::parse("div.releases>table.table>tbody>tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();

        let collections = self.document.select(&panel_selector).filter_map(|panel| {
            let title_element = panel.select(&title_selector).next().expect("Couldn't find title element");
            let title = title_element.text().collect::<String>();
            // dbg!("* {}", &title);

            let c = regex_captures!("^Microchip (SAM[A-Z0-9-]+)", &title);
            // take regex, and filter out r`Microchip (SAM[A-Z0-9]+)`
            let family = if let Some((_, sam)) = c {
                if crate::ChipsFamily::VARIANTS.contains(&sam) {
                    Rc::new(sam.to_owned())
                } else {
                    return None;
                }
            } else {
                return None;
            };

            println!("* Found {}", family);

            let chips = panel.select(&device_list_selector).map(|device_element| {
                device_element.text().collect::<String>().trim().to_owned() 
            }).collect::<Vec<String>>();
            let chips = Rc::new(chips);

            let versions = panel.select(&release_selector).filter_map(|release_element| {
                let mut column_selector = release_element.select(&td_selector);

                match column_selector.next() {
                    Some(first_column) => {
                        let text: String = first_column.text().collect();
                        let version = text.split(char::is_whitespace).next().unwrap(); // TODO: Do better error handling / consider release date
                        dbg!(&version);
                        let version = Version::parse(&version).expect("Version should not fail"); // TODO: Can we do better?
                        let _description = column_selector.next().expect("Unable to find second column");
                        let download = release_element.select(&Selector::parse("td>button.download-button").unwrap()).next().expect("Unable to find download button");
                        let archive = download.value().attr("data-link").unwrap().to_string();
                        Some(AtPack {
                            family: family.clone(),
                            version,
                            chips: chips.clone(),
                            archive,
                        })
                    },
                    None => {
                        None
                    },
                }
                
            }).collect::<Vec<AtPack>>();

            Some(AtPacksCollection {
                family: family,
                chips: chips,
                packs: versions,
            })
        }).collect::<Vec<AtPacksCollection>>();
        
        Ok(collections)
    }
}