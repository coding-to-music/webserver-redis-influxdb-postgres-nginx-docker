use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Subcommand {
    Serve(ServeOpts),
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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ServeOpts {
    #[clap(long, default_value = "3000")]
    port: u32,
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
        Subcommand::Serve(opts) => {
            gtfs::Serve::new(opts.redis_conn).serve().await;
        }
        Subcommand::Populate(opts) => {
            gtfs::Populate::new(
                opts.redis_conn,
                opts.gtfs_area,
                opts.gtfs_url,
                opts.gtfs_key,
            )
            .run()
            .await
            .unwrap();
        }
    }
}
