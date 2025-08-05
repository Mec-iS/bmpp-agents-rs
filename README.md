# Blindly Meaningful Prompting Protocol

Create programmatically any agent you need from an annotated payload, using BMPP (temporary name: Blindly Meaningful Prompting Protoocol).

This is part of the research carried on for the W3C Web Agents working group for Web Agents interoperability.

BMPP is a formal protocol to define LLM interactions for LLM systems interoperability. It is inspired by **BSPL** ([paper](https://www.cs.huji.ac.il/~jeff/aamas11/papers/A4_B57.pdf) and [related work](https://www.lancaster.ac.uk/~chopraak/publications.html)) and by **Meaning Typed Prompting**. As presented in [this paper](https://arxiv.org/pdf/2410.18146).

This repo requires [Rust](https://www.rust-lang.org/tools/install) to run.

## implementation details
* Rust has been picked because of its type system and the formal safety of its compiler, and a good trade-off between its modern features and runtime performance. It can also be easily embedded in Python via [`pyO3`](https://github.com/PyO3/pyo3), a well-supported Python bindings library.
* this repo transpile BMPP files to Rust but any language can be supported.
* it uses the [`pest` library](https://pest.rs/) for grammar management.

**TODO**:
* port all the features from https://gitlab.com/masr/bspl (formal validition)
* implement other OpenAI-compliant remote APIs
* implement transpiling to vanilla Python or LangChain/GraphChain

## Build
```
$ cargo build
```
Run tests:
```
$ cargo test
```

## Usage

### Base usage for parsing and transpiling:
```
# Parse and validate a protocol
cargo run parse protocol.bmpp --validate

# Transpile to Rust
cargo run transpile protocol.bmpp --output-dir ./generated --target rust

# Validate protocol semantics
cargo run validate protocol.bmpp --semantic-check --flow-check

# Format a protocol file
cargo run format protocol.bmpp --in-place

# Create a new protocol from template
cargo run init MyProtocol --template basic --output my_protocol.bmpp
```

### Integration with local Ollama for NL:

Need Ollama installed, see below.
```
# Convert a BMPP protocol to natural language
cargo run from-protocol protocol.bmpp --style detailed --output description.txt

# Convert natural language to BMPP protocol
cargo run to-protocol "A simple protocol where a customer requests a quote from a supplier" --output generated_protocol.bmpp

# Read from file and generate protocol with validation
cargo run to-protocol requirements.txt --input-file --max-attempts 5

# Generate without validation (useful for debugging)
cargo run to-protocol "A three-party negotiation protocol" --skip-validation

# Verbose mode for detailed output
cargo run from-protocol protocol.bmpp --verbose --style technical
```



## Install Ollama

For now this repo only support Ollama.

```
$ curl -fsSL https://ollama.com/install.sh | sh
```
Check `localhost:11434` in your browser.

### models selection

```
$ ollama pull <MODEL>
$ ollama serve
$ ollama run <MODEL>
```
Models available: [link](https://ollama.com/library).

Set:
```
export OLLAMA_MODEL=llama3.1
```
or any other model you have downloaded to change model.

## Improvements

### 1. **Path-Based Verification System** (from `verification/paths.py`)

Add support for generating and analyzing all possible execution paths of a protocol:

```rust
// Add to a new module: src/verification/mod.rs
pub mod paths;
pub mod safety;
pub mod liveness;

pub use paths::*;
pub use safety::*;
pub use liveness::*;
```


### 2. **Safety and Liveness Verification** (from `verification/mambo.py`)

Add formal safety and liveness checking:

```rust
// src/verification/safety.rs
pub fn check_safety(protocol: &Protocol) -> Result<bool> {
    // Check for parameter conflicts (multiple producers)
    // Check for deadlocks
    // Check for unreachable states
}

pub fn check_liveness(protocol: &Protocol) -> Result<bool> {
    // Ensure all paths can complete
    // Check that all parameters can be bound
}
```


### 3. **Protocol Composition and References** (from `protocol.py`)

Enhance the protocol system to support sub-protocols and composition:

```rust
// Add to AST node types
pub enum AstNodeType {
    // ... existing types
    ProtocolReference,
    ProtocolComposition,
}
```

### 4. **Enhanced CLI Commands** (from `main.py`)

Expand the CLI to include verification commands:

```rust
// Add to src/cli/args.rs
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands
    
    /// Verify protocol safety properties
    VerifySafety {
        /// Input BMPP file to verify
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
    
    /// Verify protocol liveness properties  
    VerifyLiveness {
        /// Input BMPP file to verify
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },
    
    /// Generate all possible execution paths
    GeneratePaths {
        /// Input BMPP file
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        /// Maximum path length to explore
        #[arg(long, default_value = "10")]
        max_depth: usize,
    },
}
```


### 5. **Enhanced Error Reporting** (inspired by BSPL's validation)

Add more detailed error reporting with suggestions:

```rust
// src/utils/errors.rs
#[derive(Debug)]
pub struct ValidationError {
    pub error_type: ValidationErrorType,
    pub location: Location,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug)]
pub enum ValidationErrorType {
    UndeclaredParameter,
    CausalViolation,
    SafetyViolation,
    LivenessViolation,
    TypeMismatch,
}
```


### 6. **Configuration Management** (from BSPL's adapter configs)

Enhance configuration with protocol-specific settings:

```rust
// Enhance src/config.rs
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub ollama_base_url: String,
    pub ollama_model: String,
    
    // New verification settings
    pub verification: VerificationConfig,
    pub generation: GenerationConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VerificationConfig {
    pub enable_safety_checks: bool,
    pub enable_liveness_checks: bool,
    pub max_path_depth: usize,
}
```


### 7. **Protocol Metrics and Analysis** (from `verification/`)

Add analytical capabilities:

```rust
// src/analysis/mod.rs
pub struct ProtocolMetrics {
    pub parameter_count: usize,
    pub interaction_count: usize,
    pub role_count: usize,
    pub cyclomatic_complexity: usize,
    pub max_path_length: usize,
}

pub fn analyze_protocol(protocol: &Protocol) -> ProtocolMetrics {
    // Compute various metrics about the protocol
}
```
