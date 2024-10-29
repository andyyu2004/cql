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
    #[clap(short, long, default_value = "json")]
    output: Format,
}

#[derive(Default, Clone)]
enum Format {
    #[default]
    Json,
}

impl FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "json" => Ok(Self::Json),
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

            match args.output {
                Format::Json => serde_json::to_writer(std::io::stdout(), &values)?,
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

#[cfg(feature = "serde")]
mod serde_impls;
