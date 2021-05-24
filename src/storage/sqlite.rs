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
    fn from_row(row: &Row) -> Result<Entry, rusqlite::Error> {
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
    fn lookup_entry(&self, bucket: &str, key: &str) -> Result<Option<Entry>, Error> {
        let mut stmt = self.conn.prepare(
            "select * from entries where bucket = ?1 and key = ?2"
        )?;

        let mut rows = stmt.query(params![bucket, key])?;
        let row = rows.next()?;
        match row {
            Some(row) => Entry::from_row(row).map(Some).map_err(Error::from),
            None => Ok(None),
        }
    }

    fn store_entry(&self, entry: Entry) -> Result<(), Error> {
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

    fn delete_entry(&self, bucket: &str, key: &str) -> Result<(), Error> {
        self.conn.execute("
                delete from entries where bucket = ?1 and key = ?2
            ", params![bucket, key])?;
        Ok(())
    }

    fn list_buckets(&self) -> Result<Vec<String>, Error> {
        let mut stmt = self.conn.prepare(
            "select distinct bucket from entries order by bucket"
        )?;

        let buckets = stmt
            .query_map([], |row| Ok(row.get(0)?))?
            .collect::<Result<Vec<String>, rusqlite::Error>>()?;

        Ok(buckets)
    }

    fn list_entries(&self, bucket: &str) -> Result<Vec<Entry>, Error> {
        let mut stmt = self.conn.prepare(
            "select * from entries where bucket = ?1 order by bucket"
        )?;

        let entries = stmt
            .query_map(&[bucket], Entry::from_row)?
            .collect::<Result<Vec<Entry>, rusqlite::Error>>()?;

        Ok(entries)
    }
}
