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
