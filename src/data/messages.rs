use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

/// A contact form message as persisted on disk.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContactMessage {
    pub name: String,
    pub email: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Append-only store for contact messages, backed by a JSON file.
///
/// Writes are serialized through a mutex and land on disk atomically
/// (write to a temporary file, then rename), so a crash or two concurrent
/// submissions can no longer truncate or interleave the file.
pub struct MessageStore {
    path: PathBuf,
    write_lock: Mutex<()>,
}

impl MessageStore {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join("messages.json"),
            write_lock: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn append(&self, message: &ContactMessage) -> io::Result<()> {
        let _guard = self.write_lock.lock().await;

        let mut messages = self.read_all().await?;
        messages.push(serde_json::to_value(message).map_err(invalid_data)?);

        let encoded = serde_json::to_vec_pretty(&messages).map_err(invalid_data)?;

        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let tmp_path = self.path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, &encoded).await?;
        tokio::fs::rename(&tmp_path, &self.path).await?;
        Ok(())
    }

    async fn read_all(&self) -> io::Result<Vec<Value>> {
        match tokio::fs::read(&self.path).await {
            Ok(bytes) if bytes.is_empty() => Ok(Vec::new()),
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "{} is not a valid JSON array ({}); refusing to overwrite it",
                        self.path.display(),
                        e
                    ),
                )
            }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }
}

fn invalid_data(e: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(name: &str) -> ContactMessage {
        ContactMessage {
            name: name.to_string(),
            email: "someone@example.com".to_string(),
            message: "hello".to_string(),
            timestamp: Utc::now(),
        }
    }

    #[tokio::test]
    async fn appends_messages_to_a_new_file() {
        let dir = std::env::temp_dir().join(format!("mechardo-messages-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = MessageStore::new(&dir);

        store.append(&message("first")).await.expect("first append");
        store
            .append(&message("second"))
            .await
            .expect("second append");

        let stored: Vec<ContactMessage> =
            serde_json::from_slice(&std::fs::read(store.path()).expect("read back"))
                .expect("valid json");
        assert_eq!(stored.len(), 2);
        assert_eq!(stored[0].name, "first");
        assert_eq!(stored[1].name, "second");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn refuses_to_overwrite_a_corrupt_file() {
        let dir = std::env::temp_dir().join(format!("mechardo-corrupt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create dir");
        let store = MessageStore::new(&dir);
        std::fs::write(store.path(), b"{not json").expect("seed corrupt file");

        assert!(store.append(&message("first")).await.is_err());
        assert_eq!(
            std::fs::read(store.path()).expect("file still there"),
            b"{not json"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
