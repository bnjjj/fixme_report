use std::fs::File;
use std::io::{self, prelude::*};
use std::path::PathBuf;
use std::str::FromStr;

use async_std::task;
use clap::{App, Arg};
use futures::future::join_all;

use fixme_report::config;
use fixme_report::issue_tracker;
use fixme_report::parser;

fn main() {
    let matches = App::new("fixme_report")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Command-line interface to create issue from your codebase with TODO and FIXME annotations")
        .arg(
            Arg::with_name("dry-run")
                .short("d")
                .long("dry-run")
                .help("display issues to create without creating them"),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("specify patchset file instead of parsing it via stdin"),
        )
        .arg(
            Arg::with_name("config-file")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("specify configuration file, default is fixme_settings.json in the current directory"),
        )
        .arg(
            Arg::with_name("todo-template")
                .short("t")
                .long("todo-template")
                .takes_value(true)
                .help("specify template (handlebars) file for todo cases (OPTIONAL)"),
        )
        .arg(
            Arg::with_name("fixme-template")
                .short("m")
                .long("fixme-template")
                .takes_value(true)
                .help("specify template (handlebars) file for fixme cases (OPTIONAL)"),
        )
        .get_matches();

    let mut buffer = String::new();
    if let Some(file) = matches.value_of("file") {
        let mut file = File::open(file).unwrap_or_else(|err| {
            eprintln!("error: cannot open file {} ({})", file, err);
            std::process::exit(4);
        });
        file.read_to_string(&mut buffer).expect("cannot read file");
    } else {
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("cannot read stdin");
    }

    let templates = issue_tracker::Templates {
        fixme: matches
            .value_of("fixme-template")
            .map(|val| PathBuf::from_str(val).expect("cannot convert fixme-template path")),
        todo: matches
            .value_of("todo-template")
            .map(|val| PathBuf::from_str(val).expect("cannot convert todo-template path")),
    };

    let issues = parser::parse_git_diff(buffer, &templates).unwrap_or_else(|err| {
        eprintln!("error: cannot parse git diff ({})", err);
        std::process::exit(1);
    });

    let cfg = config::load(matches.value_of("config-file")).unwrap_or_else(|err| {
        eprintln!("error: cannot parse configuration file ({})", err);
        std::process::exit(2);
    });
    let issue_tracker_client = issue_tracker::new(cfg);

    if issues.is_empty() {
        println!("no annotations found.");
    }

    let mut requests = Vec::new();
    for issue in issues {
        if !matches.is_present("dry-run") {
            requests.push(issue_tracker_client.create_issue(issue));
        } else {
            println!("+ issue to create: {:#?}", issue);
        }
    }

    if !requests.is_empty() {
        task::block_on(async {
            let issues = join_all(requests).await;
            for issue in issues {
                match issue {
                    Ok(issue) => println!(
                        "> issue created: {:#?} {}",
                        issue,
                        issue_tracker_client
                            .get_issue_url(&issue)
                            .unwrap_or_default()
                    ),
                    Err(err) => {
                        eprintln!("error: cannot create issue ({})", err);
                        std::process::exit(3);
                    }
                }
            }
        });
    }
}
