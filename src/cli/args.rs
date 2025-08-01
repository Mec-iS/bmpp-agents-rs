use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "bmpp-agents",
    about = "BMPP (Business Multi-Party Protocol) compiler and toolkit for Web Agents",
    version = "0.1.0"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Parse a BMPP protocol file and validate syntax
    Parse {
        /// Input BMPP file to parse
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Output AST as JSON for debugging
        #[arg(long)]
        output_ast: bool,
        
        /// Validate protocol semantics
        #[arg(long, default_value_t = true)]
        validate: bool,
    },
    
    /// Compile BMPP protocol to target language
    Compile {
        /// Input BMPP file to compile
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Output directory for generated code
        #[arg(short, long, default_value = "./generated")]
        output_dir: PathBuf,
        
        /// Target language for code generation
        #[arg(short, long, default_value = "rust")]
        target: String,
        
        /// Generate additional validation code
        #[arg(long)]
        include_validators: bool,
    },
    
    /// Validate a BMPP protocol file
    Validate {
        /// Input BMPP file to validate
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Check for semantic consistency
        #[arg(long, default_value_t = true)]
        semantic_check: bool,
        
        /// Validate parameter flow consistency
        #[arg(long, default_value_t = true)]
        flow_check: bool,
    },
    
    /// Format a BMPP protocol file
    Format {
        /// Input BMPP file to format
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Write formatted output back to file
        #[arg(long)]
        in_place: bool,
        
        /// Output formatted code to stdout
        #[arg(long)]
        stdout: bool,
    },
    
    /// Initialize a new BMPP protocol template
    Init {
        /// Name of the protocol
        #[arg(value_name = "PROTOCOL_NAME")]
        name: String,
        
        /// Output file for the protocol template
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Protocol template type
        #[arg(short, long, default_value = "basic")]
        template: String,
    },
}
