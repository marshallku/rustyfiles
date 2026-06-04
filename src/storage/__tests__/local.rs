#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use tempfile::tempdir;

    use crate::storage::{local::LocalStorage, Storage, StorageError};

    /// A normal key round-trips and the bytes land under the cache root.
    #[tokio::test]
    async fn put_get_roundtrip_under_root() {
        let dir = tempdir().unwrap();
        let storage = LocalStorage::new(dir.path());
        let key = "files/example.com/docs/a.txt";

        storage
            .put_bytes(key, Bytes::from_static(b"hello"))
            .await
            .expect("put should succeed for a normal key");

        assert!(storage.exists(key).await.unwrap());
        assert_eq!(&storage.get_bytes(key).await.unwrap()[..], b"hello");
        assert!(dir.path().join("files/example.com/docs/a.txt").exists());
    }

    /// A leading slash is normalized away rather than treated as an absolute
    /// path that would escape the root.
    #[tokio::test]
    async fn leading_slash_is_normalized() {
        let dir = tempdir().unwrap();
        let storage = LocalStorage::new(dir.path());

        storage
            .put_bytes("/files/h/a.txt", Bytes::from_static(b"x"))
            .await
            .expect("leading-slash key should resolve under root");
        assert!(dir.path().join("files/h/a.txt").exists());
    }

    /// `..` segments must not let a read escape the cache root.
    #[tokio::test]
    async fn traversal_read_is_rejected() {
        let dir = tempdir().unwrap();
        let storage = LocalStorage::new(dir.path());

        for key in [
            "files/h/../../../../../../../../etc/passwd",
            "../../../../etc/passwd",
            "files/../../secret",
        ] {
            assert!(
                matches!(storage.exists(key).await, Err(StorageError::NotFound)),
                "exists must reject traversal key {key:?}"
            );
            assert!(
                matches!(storage.get_bytes(key).await, Err(StorageError::NotFound)),
                "get_bytes must reject traversal key {key:?}"
            );
            assert!(
                matches!(storage.serve(key).await, Err(StorageError::NotFound)),
                "serve must reject traversal key {key:?}"
            );
        }
    }

    /// `..` must not let a write escape the cache root (no file is created
    /// outside, and the call is rejected).
    #[tokio::test]
    async fn traversal_write_is_rejected() {
        let dir = tempdir().unwrap();
        let storage = LocalStorage::new(dir.path());
        let escape = dir.path().parent().unwrap().join("pwned.txt");

        let result = storage
            .put_bytes("files/h/../../../../pwned.txt", Bytes::from_static(b"x"))
            .await;

        assert!(matches!(result, Err(StorageError::NotFound)));
        assert!(!escape.exists(), "traversal write must not create {escape:?}");
    }
}
