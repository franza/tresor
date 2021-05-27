#[macro_use]
extern crate clap;
extern crate tresor;

mod sub_commands;

fn main() {
    let matches = clap::clap_app!(tresor =>
        (version: "1.0")
        (author: "franza <franza@riseup.net>")
        (about: "Tresor - store your stuff safely")
        (@subcommand init =>
            (about: "Initializes new safe storage")
        )
        (@subcommand store =>
            (about: "Encrypts and stores the value in tresor")
            (@arg bucket: +required)
            (@arg key: +required)
            (@arg value: +required)
        )
        (@subcommand get =>
            (about: "Tries to decrypt the value and outputs it")
            (@arg bucket: +required)
            (@arg key: +required)
        )
        (@subcommand delete =>
            (about: "Removes the value from tresor")
            (@arg bucket: +required)
            (@arg key: +required)
        )
        (@subcommand buckets =>
            (about: "Displays all created buckets")
        )
        (@subcommand keys =>
            (about: "Displays keys of bucket and their respective values and tries to decrypt them")
            (@arg bucket: +required)
        )
    ).get_matches();

    let command_result = match matches.subcommand() {
        ("init", _) => sub_commands::call_init(),
        ("store", Some(sub_m)) => sub_commands::call_store(
            sub_m.value_of("bucket").expect(r#"Missing argument "bucket""#),
            sub_m.value_of("key").expect(r#"Missing argument "key""#),
            sub_m.value_of("value").expect(r#"Missing argument "value""#),
        ),
        ("get", Some(sub_m)) => sub_commands::call_get(
            sub_m.value_of("bucket").expect(r#"Missing argument "bucket""#),
            sub_m.value_of("key").expect(r#"Missing argument "key""#),
        ),
        ("delete", Some(sub_m)) => sub_commands::call_delete(
            sub_m.value_of("bucket").expect(r#"Missing argument "bucket""#),
            sub_m.value_of("key").expect(r#"Missing argument "key""#),
        ),
        ("buckets", _) => sub_commands::call_buckets_list(),
        ("keys", Some(sub_m)) => sub_commands::call_entries_list(
            sub_m.value_of("bucket").expect(r#"Missing argument "bucket""#)
        ),
        (s, _) => Ok(println!("Unknown command {}", s))
    };

    command_result.unwrap_or_else(|err| println!("{}", err))
}
