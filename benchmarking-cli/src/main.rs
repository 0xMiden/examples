mod utils_input;
mod utils_program;
use clap::{Parser, Subcommand};
use miden_air::ExecutionOptions;
use miden_vm::{DefaultHost, ProvingOptions};
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;

#[derive(Parser)]
#[clap(
    author = "Miden",
    version,
    about = "A comprehensive CLI for Miden examples (benchmarking and testing)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Benchmark a single example
    Benchmark {
        #[arg(
            short,
            long,
            help("Provide example name as in ../examples"),
            required(true)
        )]
        example: String,

        #[arg(
            short,
            long,
            help("Set to 'high' if 128-bit is needed"),
            default_value("")
        )]
        security: String,

        #[arg(
            short,
            long,
            help("Set the number of desired stack outputs"),
            default_value("1")
        )]
        output: usize,
    },
    /// Test a single example for compilation and execution
    Test {
        #[arg(
            short,
            long,
            help("Provide example name as in ../examples"),
            required(true)
        )]
        example: String,

        #[arg(short, long, help("Enable verbose output"), default_value("false"))]
        verbose: bool,

        #[arg(
            short,
            long,
            help("CI mode - minimal output, exit codes for automation"),
            default_value("false")
        )]
        ci: bool,
    },
    /// Test all examples in the examples directory
    TestAll {
        #[arg(short, long, help("Enable verbose output"), default_value("false"))]
        verbose: bool,

        #[arg(
            short = 'k',
            long,
            help("Continue testing even if some examples fail"),
            default_value("false")
        )]
        continue_on_error: bool,

        #[arg(
            short,
            long,
            help("CI mode - minimal output, exit codes for automation"),
            default_value("false")
        )]
        ci: bool,
    },
    /// Validate examples directory structure and file formats
    Validate {
        #[arg(
            short = 'v',
            long,
            help("Enable verbose output"),
            default_value("false")
        )]
        verbose: bool,

        #[arg(
            short,
            long,
            help("CI mode - minimal output, exit codes for automation"),
            default_value("false")
        )]
        ci: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Benchmark {
            example,
            security,
            output,
        } => {
            benchmark_example(&example, &security, output)?;
        }
        Commands::Test {
            example,
            verbose,
            ci,
        } => {
            let result = test_example(&example, verbose, ci);
            if ci {
                if result.is_err() {
                    exit(1);
                } else {
                    exit(0);
                }
            } else {
                result?;
            }
        }
        Commands::TestAll {
            verbose,
            continue_on_error,
            ci,
        } => {
            let result = test_all_examples(verbose, continue_on_error, ci);
            if ci {
                if result.is_err() {
                    exit(1);
                } else {
                    exit(0);
                }
            } else {
                result?;
            }
        }
        Commands::Validate { verbose, ci } => {
            let result = validate_examples_directory(verbose, ci);
            if ci {
                if result.is_err() {
                    exit(1);
                } else {
                    exit(0);
                }
            } else {
                result?;
            }
        }
    }

    Ok(())
}

fn benchmark_example(
    example_name: &str,
    security: &str,
    output: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("Benchmarking Miden example: {example_name}");
    println!("============================================================");

    // let's read the program
    let program_string = fs::read_to_string(format!("../examples/{example_name}.masm"))?;

    let input_string = fs::read_to_string(format!("../examples/{example_name}.inputs"))?;
    let mut inputs = utils_input::Inputs::new();
    inputs
        .deserialize_inputs(input_string.as_str())
        .map_err(|err| format!("Failed to deserialize inputs - {err:?}"))?;

    // Compilation time
    let now = Instant::now();
    let mut program = utils_program::MidenProgram::new(program_string.as_str());
    program
        .compile_program()
        .map_err(|err| format!("Failed to compile program - {err:?}"))?;

    println! {"Compilation Time (cold): {} ms", now.elapsed().as_millis()}

    let program_to_run = program.program.clone().unwrap();

    let _host = DefaultHost::default();

    let execution_options =
        ExecutionOptions::new(None, 64, false, false).map_err(|err| format!("{err}"))?;

    // Execution time
    let now = Instant::now();
    let advice_inputs = inputs.advice_inputs.clone();
    let mut host = DefaultHost::default();
    let trace = miden_vm::execute(
        &program_to_run,
        inputs.stack_inputs.clone(),
        advice_inputs,
        &mut host,
        execution_options,
    )
    .map_err(|err| format!("Failed to generate execution trace = {err:?}"))
    .unwrap();

    println! {"Execution Time: {} steps in {} ms", trace.get_trace_len(), now.elapsed().as_millis()}

    // Proving time
    let proof_options = if security == "high" {
        ProvingOptions::with_128_bit_security(false)
    } else {
        ProvingOptions::with_96_bit_security(false)
    };

    let advice_inputs_proof = inputs.advice_inputs.clone();
    let _host = DefaultHost::default();

    let now = Instant::now();
    let mut host_for_prove = DefaultHost::default();
    let (output_result, proof) = miden_vm::prove(
        &program.program.unwrap(),
        inputs.stack_inputs.clone(),
        advice_inputs_proof,
        &mut host_for_prove,
        proof_options,
    )
    .expect("Proving failed");

    println! {"Proving Time: {} ms", now.elapsed().as_millis()}

    // Verification time
    let program_info = program.program_info.unwrap();

    let now = Instant::now();
    miden_vm::verify(
        program_info,
        inputs.stack_inputs,
        output_result.clone(),
        proof,
    )
    .map_err(|err| format!("Program failed verification! - {err}"))?;

    println! {"Verification Time: {} ms", now.elapsed().as_millis()}

    // We return the stack as defined by the user
    println! {"Result: {:?}", output_result.stack_truncated(output)};

    Ok(())
}

fn test_example(
    example_name: &str,
    verbose: bool,
    ci: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !ci {
        println!("Testing example: {example_name}");
    }

    // Check if .masm file exists
    let masm_path = PathBuf::from(format!("../examples/{example_name}.masm"));
    if !masm_path.exists() {
        return Err(format!("Example file not found: {}", masm_path.display()).into());
    }

    // Check if .inputs file exists
    let inputs_path = PathBuf::from(format!("../examples/{example_name}.inputs"));
    if !inputs_path.exists() {
        return Err(format!("Inputs file not found: {}", inputs_path.display()).into());
    }

    // Read and validate .masm file
    let program_string =
        fs::read_to_string(&masm_path).map_err(|e| format!("Failed to read .masm file: {e}"))?;

    if verbose {
        println!("  OK .masm file read successfully");
    }

    // Read and validate .inputs file
    let input_string = fs::read_to_string(&inputs_path)
        .map_err(|e| format!("Failed to read .inputs file: {e}"))?;

    // Validate JSON syntax
    let _: serde_json::Value = serde_json::from_str(&input_string)
        .map_err(|e| format!("Invalid JSON in .inputs file: {e}"))?;

    if verbose {
        println!("  OK .inputs file is valid JSON");
    }

    // Test compilation
    let mut program = utils_program::MidenProgram::new(&program_string);
    program
        .compile_program()
        .map_err(|e| format!("Compilation failed: {e}"))?;

    if verbose || !ci {
        println!("  OK Compilation successful");
    }

    // Test execution
    let mut inputs = utils_input::Inputs::new();
    inputs
        .deserialize_inputs(&input_string)
        .map_err(|e| format!("Failed to deserialize inputs: {e}"))?;

    let advice_inputs_test = inputs.advice_inputs.clone();
    let _host = DefaultHost::default();
    let execution_options = ExecutionOptions::new(None, 64, false, false)
        .map_err(|e| format!("Failed to create execution options: {e}"))?;

    let mut host_for_test = DefaultHost::default();
    let trace = miden_vm::execute(
        &program.program.clone().unwrap(),
        inputs.stack_inputs.clone(),
        advice_inputs_test,
        &mut host_for_test,
        execution_options,
    )
    .map_err(|e| format!("Execution failed: {e}"))?;

    if verbose {
        println!(
            "  OK Execution successful ({} steps)",
            trace.get_trace_len()
        );
    }

    if !ci {
        println!("  OK Example '{example_name}' passed all tests");
    }

    Ok(())
}

fn test_all_examples(
    verbose: bool,
    continue_on_error: bool,
    ci: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !ci {
        println!("Testing all examples...");
    }

    let examples_dir = PathBuf::from("../examples");
    let mut failed_examples = Vec::new();
    let mut passed_count = 0;
    let mut total_count = 0;

    for entry in fs::read_dir(&examples_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("masm") {
            total_count += 1;
            let example_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid file name")?;

            match test_example(example_name, verbose, true) {
                Ok(_) => {
                    passed_count += 1;
                    if verbose && !ci {
                        println!("PASS {example_name}");
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    failed_examples.push((example_name.to_string(), error_msg.clone()));
                    if !ci {
                        println!("FAIL {example_name}: {error_msg}");
                    }
                    if !continue_on_error {
                        break;
                    }
                }
            }
        }
    }

    if !ci {
        println!("\nTest Summary:");
        println!("  Total examples: {total_count}");
        println!("  Passed: {passed_count}");
        println!("  Failed: {}", failed_examples.len());

        if !failed_examples.is_empty() {
            println!("\nFailed examples:");
            for (name, error) in &failed_examples {
                println!("  - {name}: {error}");
            }
        }
    }

    if failed_examples.is_empty() {
        if !ci {
            println!("OK All examples passed!");
        }
        Ok(())
    } else {
        Err(format!("{} examples failed", failed_examples.len()).into())
    }
}

fn validate_examples_directory(_verbose: bool, ci: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !ci {
        println!("Validating examples directory structure...");
    }

    let examples_dir = PathBuf::from("../examples");
    let mut validation_errors = Vec::new();

    for entry in fs::read_dir(&examples_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("masm") {
            let example_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid file name")?;

            // Check for corresponding .inputs file
            let inputs_path = path.with_extension("inputs");
            if !inputs_path.exists() {
                validation_errors.push(format!("Missing .inputs file for {example_name}"));
            }

            // Validate .masm file content
            if let Ok(content) = fs::read_to_string(&path) {
                if content.trim().is_empty() {
                    validation_errors.push(format!("Empty .masm file: {example_name}"));
                }
            } else {
                validation_errors.push(format!("Cannot read .masm file: {example_name}"));
            }

            // Validate .inputs file JSON if it exists
            if inputs_path.exists() {
                if let Ok(content) = fs::read_to_string(&inputs_path) {
                    if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                        validation_errors
                            .push(format!("Invalid JSON in .inputs file: {example_name}"));
                    }
                } else {
                    validation_errors.push(format!("Cannot read .inputs file: {example_name}"));
                }
            }
        }
    }

    if !validation_errors.is_empty() {
        for error in &validation_errors {
            if !ci {
                println!("ERROR {error}");
            }
        }
        Err(format!(
            "Directory validation failed with {} errors",
            validation_errors.len()
        )
        .into())
    } else {
        if !ci {
            println!("OK Examples directory validation passed");
        }
        Ok(())
    }
}
