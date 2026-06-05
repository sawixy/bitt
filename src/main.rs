mod protocol;

use protocol::file:: {File, Entry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut f = File::new();
    f.load(String::from("archlinux-2026.06.01-x86_64.iso.torrent")).await?;

    println!("{:?}", f);
    Ok(())
}