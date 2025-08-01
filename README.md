# Blindly Meaningful Protocol

Create programmatically any agent you need from an annotated payload, using BMP.

This is part of the research carried on for the W3C Web Agents working group for Web Agents interoperability.

BMP is a formal protocol to define LLM interactions for LLM systems interoperability. It is inspired by **BSPL** ([paper](https://www.cs.huji.ac.il/~jeff/aamas11/papers/A4_B57.pdf) and [related work](https://www.lancaster.ac.uk/~chopraak/publications.html)) and by **Meaning Typed Prompting*. As presented in [this paper](https://arxiv.org/pdf/2410.18146).


## Build
```
$ cargo build
```
Run tests:
```
$ cargo test
```


## Usage
```
# Parse and validate a protocol
bmpp-agents parse protocol.bmpp --validate

# Compile to Rust
bmpp-agents compile protocol.bmpp --output-dir ./generated --target rust

# Validate protocol semantics
bmpp-agents validate protocol.bmpp --semantic-check --flow-check

# Format a protocol file
bmpp-agents format protocol.bmpp --in-place

# Create a new protocol from template
bmpp-agents init MyProtocol --template basic --output my_protocol.bmpp
```

## Usage

It works for now it only works with Ollama but simple clients to any OpenAI-style API could be implemented.

1. Create your resource description file or string (see `examples/`)
2. `cargo run -- your_file.vibe`
3. modify the `generated/main.rs` to make the desired calls to the LLM using the pregenerated code
4. `cd generate && cargo run`. Enjoy 

## Build
```
$ cargo build
```
Run an example:
```
$ cargo run -- examples/knowledge_retrieval.vibe --output-dir ./generated
```
Run tests:
```
$ cargo test
# OR
$ cargo test --test test_unit_extra
```

### install ollama

For now this only support Ollama.

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
