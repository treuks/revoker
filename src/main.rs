use clap::Parser;

use revoker::modules::network;
use revoker::modules::verify;
/// Smol program to revoke a Twitch OAuth token.
#[derive(Parser, Debug)]
#[clap(
    author = "\n Written by TreuKS",
    version = "v1.0.0",
    about = "Allows you to revoke your Twitch OAuth Token",
    long_about = "\n Revoker is a small and compact cli tool, which allows you \n to effortlessly revoke your Twitch OAuth token\n"
)]
struct Args {
    /// Put your Twitch OAuth token here.
    #[clap(short, long)]
    token: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match verify::parse_pos_token(args.token) {
        Ok(token) => match verify::advanced_token_check(&token).await {
            Ok(good_token) => match network::revoke_token(token, good_token).await {
                Ok(revoked_token) => println!("{}", revoked_token),
                Err(err) => eprintln!("{}", err),
            },

            Err(e) => eprintln!("{}", e),
        },

        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
