use anyhow::Result;
use clap::Parser;
use futures_util::TryStreamExt;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "9042")]
    port: u16,
    #[clap(default_value = "localhost")]
    host: String,
    #[clap(short = 'c', long)]
    command: String,
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
        dbg!(row);
        Ok(())
    })
    .await?;
    Ok(())
}
