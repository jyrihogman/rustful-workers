use core::fmt;

#[derive(Debug)]
pub enum StorageError {
    KvStorageNotFound,
    KvOperationFailed,
    DbConnectionFailed,
    DbInsertFailed,
}
