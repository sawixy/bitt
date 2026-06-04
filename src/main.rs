mod protocol;

use protocol::file:: {File, Entry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut f = File::new();
    f.load(String::from("file.torrent")).await?;

    println!("{:?}", f);
    println!("{}", f.get("smth").unwrap().as_int().unwrap());
    println!("{:?}", f.get("listing").unwrap().as_list().unwrap());
    Ok(())
}