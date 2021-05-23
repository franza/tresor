use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    StorageAccessError(String),
    StorageOperationError(String),
    DataExtractError(String),
    GenericError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Error::StorageAccessError(s) => format!("Database access denied: {}", s),
            Error::StorageOperationError(s) => format!("Failed to perform database operation: {}", s),
            Error::DataExtractError(s) => format!("Possibly corrupted data: {}", s),
            Error::GenericError(s) => format!("Database error: {}", s),
        };
        f.write_str(message.as_str())
    }
}

pub trait Storage {
    fn lookup(&self, bucket: &str, key: &str) -> Result<Option<Entry>, Error>;
    fn store(&self, entry: Entry) -> Result<(), Error>;
    fn delete(&self, bucket: &str, key: &str) -> Result<(), Error>;
}

#[derive(Eq, PartialEq, Debug)]
pub struct Entry {
    pub bucket: String,
    pub key: String,
    pub value: String,

    // Does not map to DB 1:1. May be used as `created_on` or `modified_on`
    pub created_on: u64,
}

impl Entry {
    pub fn new(bucket: &str, key: &str, value: &str) -> Self {
        let created_on = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap()
            .as_secs();

        Entry {
            bucket: bucket.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            created_on,
        }
    }
}

pub mod sqlite {
    use rusqlite::{Connection, named_params, params, Row};

    use crate::storage::Entry;

    use super::{Error, Storage};

    pub struct SqliteStorage {
        conn: Connection,
    }

    impl From<rusqlite::Error> for Error {
        fn from(err: rusqlite::Error) -> Self {
            use Error::*;

            let message = err.to_string();
            match err {
                rusqlite::Error::SqliteFailure(_, _) => StorageAccessError(message),
                rusqlite::Error::SqliteSingleThreadedMode => StorageAccessError(message),
                rusqlite::Error::FromSqlConversionFailure(_, _, _) => DataExtractError(message),
                rusqlite::Error::IntegralValueOutOfRange(_, _) => DataExtractError(message),
                rusqlite::Error::Utf8Error(_) => StorageAccessError(message),
                rusqlite::Error::NulError(_) => StorageAccessError(message),
                rusqlite::Error::InvalidParameterName(_) => StorageOperationError(message),
                rusqlite::Error::QueryReturnedNoRows => StorageOperationError(message),
                rusqlite::Error::InvalidColumnIndex(_) => DataExtractError(message),
                rusqlite::Error::InvalidColumnName(_) => DataExtractError(message),
                rusqlite::Error::InvalidColumnType(_, _, _) => DataExtractError(message),
                rusqlite::Error::ToSqlConversionFailure(_) => StorageOperationError(message),
                rusqlite::Error::InvalidQuery => StorageOperationError(message),
                rusqlite::Error::MultipleStatement => StorageOperationError(message),
                rusqlite::Error::InvalidParameterCount(_, _) => StorageOperationError(message),
                _ => GenericError(message),
            }
        }
    }

    impl Entry {
        fn from_row(row: &Row) -> Result<Entry, Error> {
            let bucket = row.get("bucket")?;
            let key = row.get("key")?;
            let value = row.get("value")?;
            let created_on = row.get("created_on")?;
            let modified_on: Option<u64> = row.get("modified_on")?;

            Ok(Entry { bucket, key, value, created_on: modified_on.unwrap_or(created_on) })
        }
    }

    pub fn setup(filename: &str) -> Result<SqliteStorage, Error> {
        let conn = Connection::open(filename)?;

        conn.execute(
            "create table entries (
                id          integer primary key,
                bucket      text not null,
                key         text not null,
                value       text not null,
                created_on  integer not null,
                modified_on integer,
                unique(bucket, key)
            )", [])?;

        Ok(SqliteStorage { conn })
    }

    pub fn reset(filename: &str) -> Result<(), Error> {
        let conn = Connection::open(filename)?;
        conn.execute("drop table entries", [])?;
        Ok(())
    }

    impl SqliteStorage {
        pub fn from_db(filename: &str) -> Result<impl Storage, Error> {
            let conn = Connection::open(filename)?;
            Ok(SqliteStorage { conn })
        }
    }

    impl Storage for SqliteStorage {
        fn lookup(&self, bucket: &str, key: &str) -> Result<Option<Entry>, Error> {
            let mut stmt = self.conn.prepare(
                "select * from entries where bucket = ?1 and key = ?2"
            )?;

            let mut rows = stmt.query(params![bucket, key])?;
            let row = rows.next()?;
            match row {
                Some(row) => Entry::from_row(row).map(Some),
                None => Ok(None),
            }
        }

        fn store(&self, entry: Entry) -> Result<(), Error> {
            let updated = self.conn.execute(
                "insert into entries (bucket, key, value, created_on)
                 values (:bucket, :key, :value, :updated_on)
                 on conflict (bucket, key)
                 do update set value = :value, modified_on = :updated_on",
                named_params! {
                    ":bucket": entry.bucket,
                    ":key": entry.key,
                    ":value": entry.value,
                    ":updated_on": entry.created_on
                })?;

            match updated {
                1 => Ok(()),
                0 => panic!("failed to insert or update key {} in bucket {}", entry.key, entry.bucket),
                _ => panic!("updated more than one rows in db which may be result of inconsistency")
            }
        }

        fn delete(&self, bucket: &str, key: &str) -> Result<(), Error> {
            self.conn.execute("
                delete from entries where bucket = ?1 and key = ?2
            ", params![bucket, key])?;
            Ok(())
        }
    }
}
