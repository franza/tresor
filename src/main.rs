extern crate tresor;

use clap::{App, Arg, SubCommand};

mod sub_commands;

fn main() {
    let bucket_arg = Arg::with_name("bucket").index(1).required(true);
    let key_arg = Arg::with_name("key").index(2).required(true);
    let value_arg = Arg::with_name("value").index(3).required(true);

    let app = App::new("Tresor - store your stuff safely")
        .subcommand(SubCommand::with_name("init"))
        .subcommand(SubCommand::with_name("store")
            .arg(bucket_arg.clone())
            .arg(key_arg.clone())
            .arg(value_arg)
        )
        .subcommand(SubCommand::with_name("get")
            .arg(bucket_arg.clone())
            .arg(key_arg.clone())
        )
        .subcommand(SubCommand::with_name("delete")
            .arg(bucket_arg.clone())
            .arg(key_arg)
        )
        .subcommand(SubCommand::with_name("buckets"))
        .subcommand(SubCommand::with_name("keys")
            .arg(bucket_arg)
        );

    let matches = app.get_matches();

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
