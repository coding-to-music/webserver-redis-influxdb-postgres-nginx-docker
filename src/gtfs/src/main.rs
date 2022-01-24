#[macro_use]
extern crate log;

use clap::Parser;
use isahc::{AsyncReadResponseExt, HttpClient};
use mobc_redis::redis::{AsyncCommands, Client};
use std::{error::Error, fs::File};

use crate::model::Agency;

pub(crate) mod model;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Subcommand {
    Run,
    Populate(PopulateOpts),
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct PopulateOpts {
    #[clap(long)]
    gtfs_url: String,
    #[clap(long)]
    gtfs_key: String,
    #[clap(long)]
    gtfs_area: String,
    #[clap(long)]
    redis_conn: String,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::formatted_timed_builder()
        .parse_filters(&std::env::var("RUST_LOG").unwrap_or("info".to_string()))
        .init();
    let cmd = Subcommand::parse();

    match cmd {
        Subcommand::Run => todo!(),
        Subcommand::Populate(opts) => Populate::new(opts).run().await.unwrap(),
    }
}

struct Populate {
    client: HttpClient,
    opts: PopulateOpts,
}

impl Populate {
    fn new(opts: PopulateOpts) -> Self {
        let client = HttpClient::builder().build().unwrap();
        Self { client, opts }
    }

    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let archive = self.download_area_zip().await?;
        self.unzip_gtfs_archive(&archive)?;
        self.populate_redis_from_files().await?;

        Ok(())
    }

    async fn download_area_zip(&self) -> Result<String, Box<dyn Error>> {
        let area = &self.opts.gtfs_area;
        let uri = format!(
            "{}/gtfs/{}/{}.zip?key={}",
            self.opts.gtfs_url, area, area, self.opts.gtfs_key
        );

        let mut response = self.client.get_async(uri).await?;

        let body = response.bytes().await?;

        let file_name = format!("{area}_gtfs.zip");

        std::fs::write(&file_name, &body)?;

        Ok(file_name)
    }

    fn unzip_gtfs_archive(&self, archive: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(&archive)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            info!("writing {outpath:?}");
            let mut outfile = File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }

        Ok(())
    }

    async fn populate_redis_from_files(&self) -> Result<(), Box<dyn Error>> {
        let client = Client::open(self.opts.redis_conn.clone())?;

        let mut conn = client.get_async_connection().await?;

        // agency.txt
        let mut rdr = csv::Reader::from_path("agency.txt")?;
        for result in rdr.deserialize() {
            let agency: Agency = result?;

            conn.hset(
                "agencies",
                &agency.agency_id,
                serde_json::to_string(&agency)?,
            )
            .await?;
        }

        Ok(())
    }
}
