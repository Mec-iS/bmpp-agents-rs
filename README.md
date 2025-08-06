# Blindly Meaningful Prompting Protocol

Create programmatically any agent you need from an annotated payload, using BMPP (temporary name: Blindly Meaningful Prompting Protoocol).

This is part of the research carried on for the W3C Web Agents working group for Web Agents interoperability.

BMPP is a formal protocol to define LLM interactions for LLM systems interoperability. It is inspired by **BSPL** ([paper](https://www.cs.huji.ac.il/~jeff/aamas11/papers/A4_B57.pdf) and [related work](https://www.lancaster.ac.uk/~chopraak/publications.html)) and by **Meaning Typed Prompting**. As presented in [this paper](https://arxiv.org/pdf/2410.18146).

This repo requires [Rust](https://www.rust-lang.org/tools/install) to run.

## Overview

BMPP enables organizations to define unambiguous protocols for LLM and Web Agent interactions, ensuring interoperability between different systems while maintaining semantic clarity through natural language annotations.

## Installation

```
cargo install bmpp-agents
```

Or build from source:

```
git clone <repository-url>
cd bmpp-agents
cargo build --release
cargo test
```

## Quick Start

### 1. Create a new protocol

```
bmpp init MyProtocol --template basic
```

This creates a `myprotocol.bmpp` file with a basic template.

### 2. Validate your protocol

```
bmpp validate myprotocol.bmpp --flow-check --semantic-check
```

### 3. Generate Rust code

```
bmpp transpile myprotocol.bmpp ./output --target rust --include-validators
```

### 4. Convert protocol to natural language

```
bmpp from-protocol myprotocol.bmpp --style detailed
```

## Commands

### `bmpp parse`

Parse and analyze BMPP protocol files.

```
bmpp parse <INPUT> [OPTIONS]
```

**Options:**
- `--output-ast`: Display the Abstract Syntax Tree
- `--validate`: Run validation checks during parsing
- `--verbose`: Show detailed parsing information

**Example:**
```
bmpp parse protocol.bmpp --output-ast --validate --verbose
```

### `bmpp validate`

Comprehensive protocol validation according to BSPL standards.

```
bmpp validate <INPUT> [OPTIONS]
```

**Options:**
- `--semantic-check`: Validate protocol structure and semantics
- `--flow-check`: Validate parameter flow consistency and BSPL rules
- `--verbose`: Show detailed validation results

**Validation includes:**
- **Safety**: Multiple producers detection
- **Completeness**: Unused parameters and unreachable interactions
- **Causality**: Circular dependency detection
- **Enactability**: Protocol executability validation
- **Composition**: Protocol reference validation

**Example:**
```
bmpp validate complex-protocol.bmpp --semantic-check --flow-check --verbose
```

### `bmpp transpile`

Generate executable code from BMPP protocols.

```
bmpp transpile <INPUT> <OUTPUT_DIR> [OPTIONS]
```

**Options:**
- `--target <TARGET>`: Target language (currently supports `rust`)
- `--include-validators`: Generate validation code
- `--verbose`: Show compilation details

**Generated files:**
- `lib.rs`: Main protocol implementation
- `validator.rs`: Protocol validation logic (with `--include-validators`)
- `Cargo.toml`: Rust project configuration

**Example:**
```
bmpp transpile protocol.bmpp ./generated --target rust --include-validators
```

### `bmpp init`

Initialize new BMPP protocol from templates.

```
bmpp init <NAME> [OPTIONS]
```

**Options:**
- `--output <PATH>`: Output file path
- `--template <TYPE>`: Template type (`basic`, `multi-party`, `composition`)

**Templates:**
- **basic**: Simple two-party protocol
- **multi-party**: Three-party coordination protocol
- **composition**: Protocol with sub-protocol composition

**Example:**
```
bmpp init ShippingProtocol --template multi-party --output shipping.bmpp
```

### `bmpp format`

Format BMPP protocol files for consistency.

```
bmpp format <INPUT> [OPTIONS]
```

**Options:**
- `--in-place`: Format file in place
- `--stdout`: Output to stdout instead of file

**Example:**
```
bmpp format protocol.bmpp --in-place
```

### `bmpp from-protocol`

Convert BMPP protocols to natural language descriptions using LLM.

```
bmpp from-protocol <INPUT> [OPTIONS]
```

**Options:**
- `--output <PATH>`: Save description to file
- `--style <STYLE>`: Description style (`summary`, `detailed`, `technical`)

**Requirements:**
- Ollama running locally (`http://localhost:11434`)
- Set environment variables for LLM configuration

**Example:**
```
bmpp from-protocol protocol.bmpp --style detailed --output description.md
```

### `bmpp to-protocol`

Generate BMPP protocols from natural language descriptions using LLM.

```
bmpp to-protocol <INPUT> [OPTIONS]
```

**Options:**
- `--input-file`: Treat input as file path instead of text
- `--output <PATH>`: Save generated protocol to file
- `--skip-validation`: Skip protocol validation
- `--max-attempts <N>`: Maximum generation attempts (default: 3)

**Example:**
```
bmpp to-protocol "A purchasing protocol between buyer and seller with quote requests" --output purchase.bmpp

# Or from file:

bmpp to-protocol requirements.txt --input-file --output protocol.bmpp --max-attempts 5
```

## BMPP Protocol Syntax

### Basic Structure

```
ProtocolName <Protocol>("Description of the protocol") {
roles
RoleA <Agent>("Description of role A"),
RoleB <Agent>("Description of role B")

    parameters
        param1 <String>("Semantic meaning of parameter 1"),
        param2 <Int>("Semantic meaning of parameter 2")
    
    RoleA -> RoleB: action1 <Action>("Description of action")[out param1]
    RoleB -> RoleA: response <Action>("Description of response")[in param1, out param2]
    }
```

### Protocol Composition

```
MainProtocol <Protocol>("Main protocol with composition") {
roles
Client <Agent>("The client"),
Server <Agent>("The server"),
Processor <Agent>("The processor")

    parameters
        request <String>("The request data"),
        result <String>("The processing result")
    
    Client -> Server: initiate <Action>("Start processing")[out request]
    SubProtocol <Enactment>(Server, Processor, in request, out result)
    Server -> Client: complete <Action>("Return result")[in result]
    }

SubProtocol <Protocol>("Sub-protocol for processing") {
roles
Coordinator <Agent>("Coordinates processing"),
Worker <Agent>("Performs work")

    parameters
        input <String>("Input data"),
        output <String>("Output data")
    
    Coordinator -> Worker: process <Action>("Process data")[in input, out output]
    }
```

### Key Elements

- **Protocols**: Defined with `<Protocol>` tag and semantic descriptions
- **Roles**: Participating agents with `<Agent>` tag
- **Parameters**: Typed data with semantic annotations
- **Interactions**: Message flows between roles with parameter directions
- **Composition**: Sub-protocol invocation with `<Enactment>` tag
- **Types**: `String`, `Int`, `Float`, `Bool`
- **Directions**: `in` (input), `out` (output)

## Configuration

Set environment variables for LLM integration:

```
export OLLAMA_HOST=http://localhost:11434
export OLLAMA_MODEL=llama2  \# or your preferred model
```

## Validation Rules (BSPL Compliance)

BMPP enforces BSPL standards:

1. **Safety**: Each parameter has at most one producer
2. **Completeness**: No orphaned or unreachable parameters
3. **Causality**: No circular dependencies between interactions
4. **Enactability**: All interactions can be executed by their roles
5. **Composition**: Valid protocol references and role mappings

## Examples

See the `examples/` directory for complete protocol examples:
- Basic purchase protocol
- Multi-party logistics
- Protocol composition scenarios
- LLM agent interaction patterns

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request


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
