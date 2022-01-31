use crate::consts::*;
use crate::model::Agency;
use crate::model::*;
use isahc::{AsyncReadResponseExt, HttpClient};
use redis::{async_pool::mobc_redis::redis::AsyncCommands, async_pool::AsyncRedisPool};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashSet, error::Error, fs::File};

const GTFS_DOWNLOAD_DIR: &'static str = "gtfs_download";

pub struct Populate {
    http_client: HttpClient,
    redis_pool: AsyncRedisPool,
    gtfs_area: String,
    gtfs_url: String,
    gtfs_key: String,
}

impl Populate {
    pub async fn new(
        redis_conn: String,
        gtfs_area: String,
        gtfs_url: String,
        gtfs_key: String,
    ) -> Self {
        let client = HttpClient::builder().build().unwrap();
        let redis_pool = AsyncRedisPool::new(redis_conn);
        Self {
            http_client: client,
            redis_pool,
            gtfs_area,
            gtfs_url,
            gtfs_key,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
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
        let area = &self.gtfs_area;
        let uri = format!(
            "{}/gtfs/{}/{}.zip?key={}",
            self.gtfs_url, area, area, self.gtfs_key
        );

        let mut response = self.http_client.get_async(uri).await?;

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
        info!("populating redis...");

        self.update_hash_set_from_csv::<Agency>("agency.txt", AGENCY_REDIS_KEY)
            .await?;

        self.update_hash_set_from_csv::<Calendar>("calendar.txt", CALENDAR_REDIS_KEY)
            .await?;

        self.update_hash_set_from_csv::<Stop>("stops.txt", STOP_REDIS_KEY)
            .await?;

        self.update_hash_set_from_csv::<Route>("routes.txt", ROUTE_REDIS_KEY)
            .await?;

        // Self::update_hash_set_from_csv::<Attribution>(&mut conn, "attributions.txt", "attribution").await?;

        Ok(())
    }

    async fn update_hash_set_from_csv<T>(
        &self,
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
        self.redis_remove_old_insert_new(key_name, items).await?;
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
        &self,
        redis_key: &str,
        items: Vec<(String, T)>,
    ) -> Result<(), Box<dyn Error>>
    where
        T: Serialize,
    {
        let mut conn = self.redis_pool.get_connection().await?;
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
