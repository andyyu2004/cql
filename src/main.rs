use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use futures_util::TryStreamExt;
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
    output: Option<Format>,
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
    rows.try_for_each(|row| async {
        let _values = row.columns.into_iter().map(Value);
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
