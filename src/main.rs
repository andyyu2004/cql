use std::{io::Write, str::FromStr};

use anyhow::Result;
use clap::Parser;
use futures_util::TryStreamExt;
use indexmap::IndexMap;
use scylla::{frame::response::result::CqlValue, Session};
use serde::Serialize;

mod flatten;
mod repl;
mod value;

#[cfg(feature = "json")]
mod serde_impls;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "9042")]
    port: u16,
    #[clap(default_value = "localhost")]
    host: String,
    #[cfg(feature = "json")]
    #[clap(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(Parser)]
enum Subcommand {
    Exec(ExecArgs),
}

#[derive(Parser)]
struct ExecArgs {
    command: String,
    #[clap(short)]
    flatten: bool,
    #[clap(short, long, default_value = "json")]
    output: Format,
    #[cfg(all(feature = "csv", not(feature = "json")))]
    #[clap(short, long, default_value = "csv")]
    output: Format,
    #[cfg(not(any(feature = "json", feature = "csv")))]
    output: Format,
}

#[derive(Debug, Copy, Clone)]
enum Format {
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "json")]
    JsonPretty,
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

    match args.subcommand {
        Some(subcmd) => match subcmd {
            Subcommand::Exec(args) => exec(&sess, &args).await?,
        },
        None => repl::run(&sess).await?,
    }

    Ok(())
}

async fn exec(sess: &Session, args: &ExecArgs) -> Result<()> {
    let rows = sess.query_iter(&*args.command, ()).await?;
    let cols = rows.get_column_specs().to_vec();
    rows.map_err(anyhow::Error::from)
        .try_for_each(|row| async {
            assert_eq!(cols.len(), row.columns.len());
            // IndexMap is used to preserve the order insertion
            let values = row
                .columns
                .into_iter()
                .map(SerializableCqlValue)
                .zip(&cols)
                .map(|(v, c)| (c.name.clone(), v))
                .collect::<IndexMap<_, _>>();

            let stdout = std::io::stdout();

            if args.flatten {
                write(
                    stdout.lock(),
                    args.output,
                    flatten::flatten(serde_json::to_value(values)?),
                )?;
            } else {
                write(stdout.lock(), args.output, values)?
            };

            Ok(())
        })
        .await?;
    Ok(())
}

fn write(mut writer: impl Write, format: Format, values: impl Serialize) -> anyhow::Result<()> {
    match format {
        #[cfg(feature = "json")]
        Format::Json => serde_json::to_writer(writer, &values)?,
        #[cfg(feature = "json")]
        Format::JsonPretty => {
            serde_json::to_writer_pretty(&mut writer, &values)?;
            writeln!(&mut writer)?
        }
        #[cfg(feature = "csv")]
        Format::Csv => csv::Writer::from_writer(writer).serialize(values)?,
    }
    Ok(())
}

struct SerializableCqlValue(Option<CqlValue>);

struct SerializableCqlValueRef<'a>(Option<&'a CqlValue>);

impl<'a> SerializableCqlValueRef<'a> {
    fn new(value: &'a CqlValue) -> Self {
        Self(Some(value))
    }
}
