const VERSION: &str = env!("CARGO_PKG_VERSION");

use clap::{Arg, Command};
use oca_bundle::state::validator;
use oca_parser_xls::xls_parser::{self};

fn main() {
    let matches = Command::new("XLS(X) Parser")
        .version(VERSION)
        .subcommand(
            Command::new("parse")
            .about("Parse XLS(X) file to OCA or entries")
            .subcommand(
                Command::new("oca")
                    .about("Parse XLS(X) file to OCA")
                    .arg(
                        Arg::new("path")
                            .short('p')
                            .long("path")
                            .multiple_occurrences(true)
                            .multiple_values(true)
                            .required(true)
                            .takes_value(true)
                            .help("Path to XLS(X) file. Sample XLS(X) file can be found here: https://github.com/THCLab/oca-parser-xls/raw/main/templates/template.xlsx"),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .required(false)
                            .takes_value(true)
                            .help("Output format (json, ocafile)"),
                    )
                    /* .arg(
                        Arg::new("default-form-layout")
                            .long("default-form-layout")
                            .takes_value(false)
                            .help("Generate default Form Layout"),
                    ) */
                    .arg(
                        Arg::new("form-layout")
                            .long("form-layout")
                            .required(false)
                            .takes_value(true)
                            .help("Path to YAML file with Form Layout"),
                    )
                    /* .arg(
                        Arg::new("default-credential-layout")
                            .long("default-credential-layout")
                            .takes_value(false)
                            .help("Generate default Credential Layout"),
                    ) */
                    .arg(
                        Arg::new("credential-layout")
                            .long("credential-layout")
                            .required(false)
                            .takes_value(true)
                            .help("Path to YAML file with Credential Layout"),
                    )
                    .arg(
                        Arg::new("no-validation")
                            .long("no-validation")
                            .takes_value(false)
                            .help("Disables OCA validation"),
                    )
                    .arg(
                        Arg::new("xls-data-entry")
                            .long("xls-data-entry")
                            .takes_value(false)
                            .help("Generate OCA data entry xls file"),
                    ),
            )
            .subcommand(
                Command::new("entries")
                    .about("Parse XLS(X) file to entries")
                    .arg(
                        Arg::new("path")
                            .short('p')
                            .long("path")
                            .required(true)
                            .takes_value(true)
                            .help("Path to XLS(X) file. Sample XLS(X) file can be found here: https://github.com/THCLab/oca-rs/blob/main/tests/assets/entries_template.xlsx"),
                    ),
            )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("parse") {
        if let Some(matches) = matches.subcommand_matches("oca") {
            let validation = !matches.is_present("no-validation");
            let with_data_entry = matches.is_present("xls-data-entry");
            let paths: Vec<&str> = matches.values_of("path").unwrap().collect();
            let output_format: &str = matches.value_of("output").unwrap_or("json");
            let first_path = paths.first().unwrap().to_string();
            let mut parsed_oca_list = vec![];
            let mut parsed_oca_bundles = vec![];
            let mut errors: Vec<String> = vec![];

            for (i, p) in paths.iter().enumerate() {
                let path = p.to_string();
                let form_layout_path: Option<&str> = if i == 0 {
                    matches.value_of("form-layout")
                } else {
                    None
                };
                let credential_layout_path: Option<&str> = if i == 0 {
                    matches.value_of("credential-layout")
                } else {
                    None
                };

                let result = xls_parser::oca::parse(
                    path.clone(),
                    false, // matches.is_present("default-form-layout"),
                    form_layout_path,
                    false, // matches.is_present("default-credential-layout"),
                    credential_layout_path,
                );

                if let Err(e) = result {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({ "errors": e })).unwrap()
                    );
                    return;
                }

                let parsed = result.unwrap();
                parsed_oca_list.push(parsed.oca)
            }

            parsed_oca_list.reverse();
            let mut root_oca = parsed_oca_list.pop().unwrap();

            for mut oca in parsed_oca_list {
                /*
                let sai = oca_builder.oca.capture_base.said.clone();
                root_oca_builder.add_form_layout_reference(
                    sai.clone(),
                    oca_builder.build_default_form_layout(),
                );
                oca_builder.form_layout = Some(String::new());
                root_oca_builder.add_credential_layout_reference(
                    sai.clone(),
                    oca_builder.build_default_credential_layout(),
                );
                oca_builder.credential_layout = Some(String::new());
                */
                parsed_oca_bundles.push(oca.generate_bundle());
            }

            parsed_oca_bundles.push(root_oca.generate_bundle());
            parsed_oca_bundles.reverse();

            if validation {
                for oca_bundle in &parsed_oca_bundles {
                    let validator = validator::Validator::new();
                    let validation_result = validator.validate(oca_bundle);
                    if let Err(errs) = validation_result {
                        for e in errs {
                            errors.push(e.to_string());
                        }
                    }
                }
            }

            if errors.is_empty() {
                let filename = first_path
                    .split('/')
                    .collect::<Vec<&str>>()
                    .pop()
                    .unwrap()
                    .rsplit('.')
                    .collect::<Vec<&str>>()
                    .pop()
                    .unwrap()
                    .to_string();

                if with_data_entry {
                    match xls_parser::data_entry::generate(&parsed_oca_bundles, filename.clone()) {
                        Ok(_) => {
                            println!("OCA Data Entry written to {filename}-data_entry.xlsx");
                        }
                        Err(e) => println!("{e:?}"),
                    }
                }

                match output_format {
                    "json" => {
                        let v = serde_json::to_value(&parsed_oca_bundles).unwrap();
                        println!("{v}");
                    }
                    "ocafile" => {
                        for oca_bundle in &parsed_oca_bundles {
                            let ast = oca_bundle.to_ast();
                            let ocafile = oca_file::ocafile::generate_from_ast(&ast);
                            println!("{ocafile}");
                        }
                    }
                    _ => {
                        println!("Invalid output format");
                    }
                }
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "errors": errors })).unwrap()
                );
            }
        }

        if let Some(matches) = matches.subcommand_matches("entries") {
            let path = matches.value_of("path").unwrap().to_string();
            let result = xls_parser::entries::parse(path.clone());

            if let Err(e) = result {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "errors": e })).unwrap()
                );
                return;
            }

            let parsed = result.unwrap();

            let v = serde_json::to_value(parsed).unwrap();
            println!("{v}");
        }
    }
}
