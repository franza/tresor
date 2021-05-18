extern crate tresor;

use clap::{App, Arg, SubCommand};

use tresor::sub_commands;

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
            .arg(bucket_arg)
            .arg(key_arg)
        );

    let matches = app.get_matches();

    match matches.subcommand() {
        ("init", _) => sub_commands::call_init(),
        ("store", Some(sub_m)) => sub_commands::call_store(
            sub_m.value_of("bucket").expect("Missing argument 'bucket'"),
            sub_m.value_of("key").expect("Missing argument 'key'"),
            sub_m.value_of("value").expect("Missing argument 'value'"),
        ),
        ("get", Some(sub_m)) => sub_commands::call_get(
            sub_m.value_of("bucket").expect("Missing argument 'bucket'"),
            sub_m.value_of("key").expect("Missing argument 'key'"),
        ),
        ("delete", Some(sub_m)) => sub_commands::call_delete(
            sub_m.value_of("bucket").expect("Missing argument 'bucket'"),
            sub_m.value_of("key").expect("Missing argument 'key'"),
        ),
        (s, _) => println!("Unknown command {}", s)
    }
}
