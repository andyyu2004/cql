use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use futures_util::TryStreamExt;
use indexmap::IndexMap;
use scylla::frame::response::result::CqlValue;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "9042")]
    port: u16,
    #[clap(default_value = "localhost")]
    host: String,
    #[clap(short = 'c', long)]
    command: String,
    #[cfg(feature = "json")]
    #[clap(short, long, default_value = "json")]
    output: Format,
    #[cfg(all(feature = "csv", not(feature = "json")))]
    #[clap(short, long, default_value = "csv")]
    output: Format,
    #[cfg(not(any(feature = "json", feature = "csv")))]
    output: Format,
}

#[derive(Clone)]
enum Format {
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "csv")]
    Csv,
}

impl FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            #[cfg(feature = "json")]
            "json" => Ok(Self::Json),
            #[cfg(feature = "csv")]
            "csv" => Ok(Self::Csv),
            _ => Err(anyhow::anyhow!("unknown format: {s}")),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tokio::spawn(run()).await?
}

async fn run() -> Result<()> {
    let args = Args::parse();
    let sess = scylla::SessionBuilder::new()
        .known_node(format!("{}:{}", args.host, args.port))
        .build()
        .await?;

    let rows = sess.query_iter(&*args.command, ()).await?;
    let cols = rows.get_column_specs().to_vec();
    rows.map_err(anyhow::Error::from)
        .try_for_each(|row| async {
            assert_eq!(cols.len(), row.columns.len());
            // IndexMap is used to preserve the order insertion
            let values = row
                .columns
                .into_iter()
                .map(Value)
                .zip(&cols)
                .map(|(v, c)| (c.name.clone(), v))
                .collect::<IndexMap<_, _>>();

            let stdout = std::io::stdout();
            match args.output {
                #[cfg(feature = "json")]
                Format::Json => serde_json::to_writer(stdout, &values)?,
                #[cfg(feature = "csv")]
                Format::Csv => csv::Writer::from_writer(stdout).serialize(values)?,
            }
            Ok(())
        })
        .await?;
    Ok(())
}

struct Value(Option<CqlValue>);

struct ValueRef<'a>(Option<&'a CqlValue>);

impl<'a> ValueRef<'a> {
    fn new(value: &'a CqlValue) -> Self {
        Self(Some(value))
    }
}

#[cfg(feature = "json")]
mod flatten;

#[cfg(feature = "json")]
mod serde_impls;
