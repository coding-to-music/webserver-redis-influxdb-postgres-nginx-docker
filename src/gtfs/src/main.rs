#[macro_use]
extern crate log;

use crate::model::Agency;
use clap::Parser;
use isahc::{AsyncReadResponseExt, HttpClient};
use mobc_redis::redis::{aio::Connection, AsyncCommands, Client};
use model::{Calendar, Id, Route, Stop};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashSet, error::Error, fs::File};

pub(crate) mod model;

const GTFS_DOWNLOAD_DIR: &'static str = "gtfs_download";

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
        self.create_download_dir()?;
        std::env::set_current_dir(GTFS_DOWNLOAD_DIR)?;
        let archive = self.download_area_zip().await?;
        self.unzip_gtfs_archive(&archive)?;
        self.populate_redis_from_files().await?;

        Ok(())
    }

    fn create_download_dir(&self) -> Result<(), Box<dyn Error>> {
        if let Err(e) = std::fs::create_dir(GTFS_DOWNLOAD_DIR) {
            match e.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    info!("{GTFS_DOWNLOAD_DIR} directory already exists")
                }
                _ => return Err(Box::new(e)),
            }
        }
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
        info!("populating redis...");

        Self::update_hash_set_from_csv::<Agency>(&mut conn, "agency.txt", "agency").await?;

        Self::update_hash_set_from_csv::<Calendar>(&mut conn, "calendar.txt", "calendar").await?;

        Self::update_hash_set_from_csv::<Stop>(&mut conn, "stops.txt", "stop").await?;

        Self::update_hash_set_from_csv::<Route>(&mut conn, "routes.txt", "route").await?;

        // Self::update_hash_set_from_csv::<Attribution>(&mut conn, "attributions.txt", "attribution").await?;

        Ok(())
    }

    async fn update_hash_set_from_csv<T>(
        conn: &mut Connection,
        path: &str,
        key_name: &str,
    ) -> Result<(), Box<dyn Error>>
    where
        T: DeserializeOwned + Serialize + Id<Output = String>,
    {
        info!("reading {path} and updating contents of {key_name} in Redis...");
        let mut items = Vec::new();
        for item in Self::csv_get_generic::<T>(path)? {
            let id = item.id();
            items.push((id, item));
        }
        Self::redis_remove_old_insert_new(conn, key_name, items).await?;
        Ok(())
    }

    fn csv_get_generic<T>(path: &str) -> Result<Vec<T>, Box<dyn Error>>
    where
        T: DeserializeOwned,
    {
        info!("{path}");
        let mut rdr = csv::Reader::from_path(path)?;
        let mut v = Vec::new();
        for result in rdr.deserialize() {
            let item: T = result?;
            v.push(item);
        }

        Ok(v)
    }

    async fn redis_remove_old_insert_new<T>(
        conn: &mut Connection,
        redis_key: &str,
        items: Vec<(String, T)>,
    ) -> Result<(), Box<dyn Error>>
    where
        T: Serialize,
    {
        let mut current_ids: HashSet<String> = conn.hkeys(redis_key).await?;

        for (id, _item) in &items {
            current_ids.remove(id);
        }

        // current_ids now contains all the ids that should be removed after inserting the new ones
        let to_be_removed = current_ids;

        let mut serialized = Vec::with_capacity(items.len());
        for (id, item) in items {
            let ser = serde_json::to_string(&item)?;
            serialized.push((id, ser));
        }

        conn.hset_multiple(redis_key, &serialized).await?;

        conn.hdel(redis_key, to_be_removed).await?;

        Ok(())
    }
}
