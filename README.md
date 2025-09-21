# JVM Thread Analyzer (thread-lens)

## Overview

`thread-lens` is a Rust-based command-line interface (CLI) tool designed to analyze JVM thread dumps. It provides a normalized, JVM-agnostic view of thread dumps and can identify chronically blocked application threads by analyzing a sequence of dumps over time. This tool aims to automate the tedious and error-prone manual process of analyzing thread dumps, providing quick insights into application performance issues like deadlocks and thread starvation.

## Features

*   **Normalized Thread Dump View:** Parses raw `jstack` output from various JVMs (OpenJDK, IBM) into a consistent, structured format.
*   **Thread Categorization:** Automatically classifies stack frames and threads into JVM, Framework, and Application categories.
*   **Temporal Analysis:** Identifies threads that are persistently in a `BLOCKED` state across multiple thread dumps, highlighting potential performance bottlenecks or deadlocks in your application code.
*   **Flexible Output:** View single thread dumps in human-readable text, JSON, or YAML formats.

## Getting Started

### Prerequisites

To build and run `thread-lens`, you need to have [Rust](https://www.rust-lang.org/tools/install) and Cargo (Rust's package manager) installed on your system.

### Building the Project

Navigate to the root directory of the project and build the release version:

```bash
cargo build --release
```

This will compile the `thread-lens` library and the `thread-lens-cli` executable. The executable will be located at `target/release/thread-lens-cli`.

## Usage

All commands are run using the `thread-lens-cli` executable. You can run it directly from the `target/release/` directory or add it to your system's PATH.

### 1. Analyzing Multiple Thread Dumps (`analyze` command)

This command is used to analyze a directory containing multiple `jstack` files to identify chronically blocked application threads.

```bash
target/release/thread-lens-cli analyze --path <path_to_dump_directory>
```

Replace `<path_to_dump_directory>` with the path to the directory containing your `.jstack` files.

**Example:**

```bash
target/release/thread-lens-cli analyze --path jstack_dumps/java-app-openjdk11_20250921085315_deadlock/
```

### 2. Viewing a Single Thread Dump (`view` command)

This command allows you to view a single `jstack` file in a normalized format. You can specify the output format.

```bash
target/release/thread-lens-cli view --path <path_to_single_jstack_file> [--output <format>]
```

Replace `<path_to_single_jstack_file>` with the path to your `.jstack` file.

**Output Formats:**

*   `text` (default): Human-readable, formatted text.
*   `json`: JSON format.
*   `yaml`: YAML format.

**Examples:**

*   **Text Output (Default):**

    ```bash
    target/release/thread-lens-cli view --path jstack_dumps/java-app-openjdk11_20250921085315_block/java-app-openjdk11_20250921155316031.jstack
    ```

*   **JSON Output:**

    ```bash
    target/release/thread-lens-cli view --path jstack_dumps/java-app-openjdk11_20250921085315_block/java-app-openjdk11_20250921155316031.jstack --output json
    ```

*   **YAML Output:**

    ```bash
    target/release/thread-lens-cli view --path jstack_dumps/java-app-openjdk11_20250921085315_block/java-app-openjdk11_20250921155316031.jstack --output yaml
    ```

## Hardcoded Assumptions

The current version of `thread-lens` makes some hardcoded assumptions about JVM internals, common frameworks, and how application code is identified. These are primarily defined by regular expressions within the `thread-lens` library:

*   **JVM Patterns:** Identifies stack frames belonging to the JVM (e.g., `java.`, `sun.`, `jdk.` packages).
*   **Framework Patterns:** Identifies stack frames belonging to known frameworks (e.g., `org.eclipse.jetty.`, `spark.` packages).
*   **Application Code:** Any stack frame not identified as JVM or Framework is categorized as Application code.

Future versions could introduce configuration options to allow users to customize these patterns.
