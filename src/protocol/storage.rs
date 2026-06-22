use tokio::io::{AsyncSeekExt, AsyncWriteExt, AsyncReadExt, SeekFrom};
use tokio::fs::{File, OpenOptions};
use tokio::sync::Mutex as TokioMutex;
use std::sync::{Arc, Mutex as StdMutex, RwLock};
use std::path::Path;
use super::bitfield::BitField;

pub trait Storage: Clone {
    fn get_bitfield(&self) -> BitField;
    async fn get_piece(&self, index: usize) -> Vec<u8>;
    async fn set_piece(&mut self, index: usize, piece: Vec<u8>);
}

#[derive(Clone)]
pub struct FileStorage {
    file: Arc<TokioMutex<File>>,
    piece_length: u64,
    bitfield: Arc<StdMutex<BitField>>,
}

impl FileStorage {
    pub async fn create<P: AsRef<Path>>(path: P, total_size: u64, piece_length: u64, total_pieces: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?;
        file.set_len(total_size).await?;
        
        Ok(Self {
            file: Arc::new(TokioMutex::new(file)),
            piece_length,
            bitfield: Arc::new(StdMutex::new(BitField::new(total_pieces))),
        })
    }
}

impl Storage for FileStorage {
    fn get_bitfield(&self) -> BitField {
        self.bitfield.lock().unwrap().clone()
    }

    async fn get_piece(&self, index: usize) -> Vec<u8> {
        let offset = index as u64 * self.piece_length;
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(offset)).await.unwrap();
        
        let mut buf = vec![0u8; self.piece_length as usize];
        match file.read_exact(&mut buf).await {
            Ok(_) => buf,
            Err(_) => buf,
        }
    }

    async fn set_piece(&mut self, index: usize, piece: Vec<u8>) {
        let offset = index as u64 * self.piece_length;
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(offset)).await.unwrap();
        file.write_all(&piece).await.unwrap();
        
        self.bitfield.lock().unwrap().set_bit(index, 1);
    }
}