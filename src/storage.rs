use std::fmt;

pub struct Error(String);

impl Error {
    pub fn entry_not_found() -> Self {
        Error("entry not found".to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(format!("Failed to perform operation with storage: {}", self.0).as_str())
    }
}

pub trait Storage {
    fn lookup(&self, bucket: &str, key: &str) -> Result<String, Error>;
    fn store(&self, bucket: &str, key: &str, value: &str) -> Result<(), Error>;
    fn delete(&self, bucket: &str, key: &str) -> Result<(), Error>;
}

pub mod sqlite {
    use rusqlite::{Connection, named_params, params};
    use super::{Storage, Error};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct SqliteStorage {
        conn: Connection,
    }

    impl From<rusqlite::Error> for Error {
        fn from(err: rusqlite::Error) -> Self {
            Error(err.to_string())
        }
    }

    pub fn setup(filename: &str) -> Result<SqliteStorage, Error> {
        let conn = Connection::open(filename)?;

        conn.execute(
            "create table entries if not exist (
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
        conn.execute("truncate table entries", [])?;
        Ok(())
    }

    impl Storage for SqliteStorage {
        fn lookup(&self, bucket: &str, key: &str) -> Result<String, Error> {
            let mut stmt = self.conn.prepare(
                "select * from entries where bucket = ?1 and key = ?2"
            )?;

            let mut rows = stmt.query(params![bucket, key])?;
            let row = rows.next()?.ok_or(Error::entry_not_found())?;
            let value: String = row.get("value")?;

            Ok(value)
        }

        fn store(&self, bucket: &str, key: &str, value: &str) -> Result<(), Error> {
            let updated_on = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let updated = self.conn.execute(
                "insert into entries (bucket, key, value, created_on)
                 values (:bucket, :key, :value, :updated_on)
                 on conflict (bucket, key)
                 do update set value = :value, modified_on = :updated_on",
                named_params! {
                    ":bucket": bucket,
                    ":key": key,
                    ":value": value,
                    ":updated_on": updated_on
                })?;

            match updated {
                1 => Ok(()),
                0 => panic!("failed to insert or update key {} in bucket {}", key, bucket),
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
