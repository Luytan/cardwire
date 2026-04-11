mod args;
mod dbus;
mod output;
use args::{Args, CliMode, Commands};
use clap::{CommandFactory, Parser};
use dbus::DaemonClient;

const BIN_NAME: &str = "cardwire";

fn handle_error(err: zbus::Error) {
    match err {
        zbus::Error::MethodError(name, description, _) => {
            eprintln!("{}", description.unwrap_or_else(|| name.to_string()))
        }
        zbus::Error::FDO(fdo_err) => match *fdo_err {
            zbus::fdo::Error::ServiceUnknown(content) => {
                eprint!("error: {} \n is the service up?", content)
            }
            other => eprintln!("FDO error: {}", other),
        },
        e => eprintln!("error: {e:?}"),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    /*
       Handle completion before connecting to dbus
    */

    if let Commands::Completion { shell } = args.command {
        let mut cmd = Args::command();
        clap_complete::generate(shell, &mut cmd, BIN_NAME, &mut std::io::stdout());
        return Ok(());
    }
    let connection: zbus::Connection = zbus::connection::Builder::system()?.build().await?;
    let client: DaemonClient<'_> = DaemonClient::connect(&connection).await?;

    match args.command {
        Commands::Set { mode } => {
            let mode_string = match mode {
                CliMode::Integrated => "integrated".to_string(),
                CliMode::Hybrid => "hybrid".to_string(),
                CliMode::Manual => "manual".to_string(),
            };

            match client.set_mode(mode_string).await {
                Ok(response) => println!("{}", response),
                Err(e) => handle_error(e),
            };
        }
        Commands::Get => {
            let response_result = client.get_mode().await;
            match response_result {
                Ok(response) => eprintln!("{}", response),
                Err(e) => handle_error(e),
            };
        }
        Commands::List { full: _, json: _ } => {
            let mut response: Vec<(u32, String, String, String, bool, bool)> =
                client.list_gpus().await?;
            response.sort_by_key(|row| row.0);
            output::print_gpu_table(&response);
        }
        Commands::Gpu { id, action } => {
            match client.set_gpu_block(id, action.block).await {
                Ok(response) => println!("{}", response),
                Err(e) => handle_error(e),
            };
        }
        _ => {}
    }

    Ok(())
}
