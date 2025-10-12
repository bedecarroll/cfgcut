use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::{ArgAction, Parser};

use cfgcut::{
    Anonymization, CfgcutError, CommentHandling, OutputMode, RunRequest, TokenDestination,
    TokenRecord, run,
};

#[derive(Parser, Debug)]
#[command(
    name = "cfgcut",
    about = "Extract configuration sections from text files.",
    long_about = None,
    after_help = "Match segments are separated with '||' and implicitly anchored. Append '|>>|' to include descendant nodes, use '|#|' for comments with -c/--with-comments, and enable -a/--anonymize to scramble sensitive tokens while keeping structure intact.",
    version
)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "CLI flags map directly to user-facing switches"
)]
struct Cli {
    /// Configuration match expressions to apply, supports hierarchical syntax
    #[arg(
        short = 'm',
        long = "match",
        action = ArgAction::Append,
        value_name = "MATCH"
    )]
    matches: Vec<String>,

    /// Include comments in the output stream
    #[arg(short = 'c', long = "with-comments")]
    with_comments: bool,

    /// Suppress output and return success on match
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Scramble sensitive values like usernames, passwords, ASNs, and IPs
    #[arg(short = 'a', long = "anonymize")]
    anonymize: bool,

    /// Emit matched tokens as JSON lines on stdout
    #[arg(long = "tokens", action = ArgAction::SetTrue)]
    tokens: bool,

    /// Write matched tokens to a file (implies --tokens)
    #[arg(long = "tokens-out")]
    tokens_out: Option<PathBuf>,

    /// Input configuration files or directories
    #[arg(value_name = "PATH", required = true)]
    inputs: Vec<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let Cli {
        matches,
        with_comments,
        quiet,
        anonymize,
        tokens,
        tokens_out,
        inputs,
    } = cli;

    let request = RunRequest::builder()
        .matches(matches)
        .comment_handling(if with_comments {
            CommentHandling::Include
        } else {
            CommentHandling::Exclude
        })
        .output_mode(if quiet {
            OutputMode::Quiet
        } else {
            OutputMode::Normal
        })
        .anonymization(if anonymize {
            Anonymization::Enabled
        } else {
            Anonymization::Disabled
        })
        .inputs(inputs)
        .token_output(
            tokens_out
                .map(TokenDestination::File)
                .or_else(|| tokens.then_some(TokenDestination::Stdout)),
        )
        .build();

    match run(&request) {
        Ok(result) => {
            for warning in &result.warnings {
                eprintln!("{warning}");
            }
            if !result.matched {
                std::process::exit(1);
            }
            if !matches!(request.output_mode(), OutputMode::Quiet) {
                print!("{}", result.stdout);
            }
            if let Some(dest) = request.token_output()
                && let Err(err) = write_tokens(dest, &result.tokens)
            {
                report_error(&err);
                std::process::exit(1);
            }
        }
        Err(err) => {
            report_error(&err);
            std::process::exit(1);
        }
    }
}

fn report_error(err: &CfgcutError) {
    eprintln!("{err}");
}

fn write_tokens(dest: &TokenDestination, tokens: &[TokenRecord]) -> Result<(), CfgcutError> {
    if tokens.is_empty() {
        return Ok(());
    }
    match dest {
        TokenDestination::Stdout => {
            for record in tokens {
                let line = serde_json::to_string(record).map_err(CfgcutError::from)?;
                println!("{line}");
            }
            Ok(())
        }
        TokenDestination::File(path) => {
            let mut file = File::create(path).map_err(|source| CfgcutError::Io {
                path: path.clone(),
                source,
            })?;
            for record in tokens {
                let line = serde_json::to_string(record).map_err(CfgcutError::from)?;
                file.write_all(line.as_bytes())
                    .map_err(|source| CfgcutError::Io {
                        path: path.clone(),
                        source,
                    })?;
                file.write_all(b"\n").map_err(|source| CfgcutError::Io {
                    path: path.clone(),
                    source,
                })?;
            }
            Ok(())
        }
        _ => Err(CfgcutError::UnsupportedTokenDestination),
    }
}
