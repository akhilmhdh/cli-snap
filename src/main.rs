use ansi_term::Color::{Blue, Green, Red};
use clap::Parser;
use serde_derive::Deserialize;
use shell_words::split;
use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    path::PathBuf,
    process::{exit, Command},
};
use toml;

#[derive(Deserialize)]
struct TestPlan {
    tests: Vec<Test>,
    config: Config,
}

#[derive(Deserialize)]
struct Test {
    commands: Vec<String>,
    id: String,
}

#[derive(Deserialize)]
struct Config {
    snapshot_directory: PathBuf,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value=get_current_working_dir().into_os_string())]
    config: PathBuf,

    #[arg(long = "update-snapshot", default_value_t = false)]
    is_snapshot_update: bool,
}

fn get_current_working_dir() -> PathBuf {
    env::current_dir().unwrap()
}

fn main() {
    let cli = Cli::parse();

    let mut cfg_file_path = cli.config.clone();
    cfg_file_path.push("cli-snap.toml");

    let test_plan_file = match fs::read_to_string(cfg_file_path.clone()) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Unable to find cli-snap config file in {:?}", cfg_file_path);
            exit(1);
        }
    };

    let test_plan: TestPlan = match toml::from_str(&test_plan_file) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Invalid cli-snap config file");
            eprintln!("{}", err);
            exit(1);
        }
    };

    let snap_dir = PathBuf::from(cli.config).join(test_plan.config.snapshot_directory);
    fs::create_dir_all(&snap_dir).unwrap();

    let mut new_snapshots_set: HashMap<String, String> = HashMap::new();

    for test in &test_plan.tests {
        let mut final_output = String::new();

        for cmd in &test.commands {
            let args: Vec<_> = split(cmd).expect("Failed to split command");
            let command = &args[0];

            let out = Command::new(command)
                .args(&args[1..])
                .output()
                .expect("Failed to write command");

            final_output = String::from_utf8_lossy(&out.stdout).to_string();
            if !out.status.success() {
                eprintln!(
                    "Failed to execute command, {:?}",
                    String::from_utf8_lossy(&out.stderr)
                );
                exit(1)
            }
        }
        new_snapshots_set.insert(test.id.clone(), final_output);
    }

    let mut exit_code = 0;

    let mut passed_tests = vec![];
    let mut failed_tests = vec![];

    for test in &test_plan.tests {
        println!("- Running snapshot test {}.", test.id);

        let mut snapshot_file = PathBuf::from(snap_dir.clone());
        snapshot_file.push(format!("{}.txt", test.id));

        let old_snapshot = fs::read_to_string(&snapshot_file).unwrap_or_default();
        let new_snapshot = match new_snapshots_set.get(&test.id) {
            Some(c) => c,
            None => {
                eprintln!("Missing snapshot for {}", test.id);
                exit(0);
            }
        };

        if old_snapshot != *new_snapshot {
            println!(
                "{} does not match with {}",
                Red.paint("Received snapshot"),
                Green.paint(format!("stored snapshot {}.", test.id))
            );
            println!("{}", Green.paint(format!("Snapshot:\n{}", old_snapshot)));
            println!("{}", Red.paint(format!("Received:\n{}", new_snapshot)));
            failed_tests.push(test.id.clone());
            exit_code = 1;
        } else {
            passed_tests.push(test.id.clone());
        }

        if cli.is_snapshot_update {
            println!("Updating snapshot - {}", test.id);

            let mut file = match fs::File::create(&snapshot_file) {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error creating file: {}", e);
                    return;
                }
            };

            match file.write_all(new_snapshot.as_bytes()) {
                Ok(_) => Some(true),
                Err(e) => {
                    eprintln!("Error writing to file: {}", e);
                    exit(1);
                }
            };
        }
    }

    let total_tests = test_plan.tests.len();
    println!("{}", Blue.underline().paint("Test Summary"));
    println!("Total Tests: {}", Blue.paint(total_tests.to_string()));
    println!("Passed: {}", Green.paint(passed_tests.len().to_string()));
    println!("Failed: {}", Red.paint(failed_tests.len().to_string()));

    exit(exit_code);
}
