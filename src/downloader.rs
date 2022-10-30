use anyhow::Error;
use bytes::Bytes;
use reqwest::{Url, Client};

pub struct Downloader {
    address: Url,
    client: Client,
}

impl Downloader {
    pub fn new(address: Url) -> Result<Downloader, Error> {
        let client = Client::builder()
            .cookie_store(true)
            .gzip(true)
            .deflate(true)
            .connection_verbose(false) // TODO: handle if high enough -v is passed
            .build()?;
        Ok(Downloader {
            address,
            client,
        })
    }
    
    /// Loads repository's main page for further processing
    pub async fn load_repository(&self) -> Result<String, Error> {
        let response = self.client.get(self.address.clone())
            .send().await?;
        let text = response.text().await?;
        Ok(text)
    }

    pub async fn load_file(&self, file: &str) -> Result<Bytes, Error> {
        let address = self.address.join(file)?;
        let response = self.client.get(address).send().await?;
        let content = response.bytes().await?;
        Ok(content)
    }
}