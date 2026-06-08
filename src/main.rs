mod protocol;
mod client;
use crate::client::app::render;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    render().await?;

    Ok(())
}


