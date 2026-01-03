# Anagram Solver

A high-performance web application that finds word combinations (anagrams) for any given phrase. It utilizes a **Rust-based recursive search algorithm** compiled to **WebAssembly (WASM)** for efficient processing directly in the browser.

## Features

* **Fast Recursive Search**: Efficiently traverses possible word combinations to find valid anagrams.
* **Quality Scoring**: Results are ranked based on a quality score that prefers fewer, longer, and more balanced word lengths.
* **Redundancy Filtering**: Uses canonical signatures to prevent showing redundant word combinations.
* **Embedded Dictionary**: Includes a comprehensive built-in dictionary (`dict.txt`) for word verification.
* **Modern Web UI**: A clean, responsive interface for entering phrases and viewing results.

## Technology Stack

* **Rust**: Core logic and search algorithm.
* **WebAssembly**: High-speed execution within the browser.
* **wasm-bindgen & Serde**: Seamless communication between JavaScript and Rust.
* **HTML/CSS/JavaScript**: Frontend presentation and WASM module initialization.

## Project Structure

* `src/lib.rs`: The core solver logic, including frequency computation and the recursive search algorithm.
* `index.html`: The main user interface and entry point for the application.
* `dict.txt`: The text file containing the word list used for anagram generation.
* `pkg/`: Directory containing the compiled WebAssembly binaries and JavaScript glue code.
* `Cargo.toml`: Rust project configuration and dependencies.

## Setup and Installation

### Prerequisites

* [Rust](https://www.rust-lang.org/) and `cargo`
* [`wasm-pack`](https://drager.github.io/wasm-pack/) for building the WASM module

### Building the Project

1. Compile the Rust code to WebAssembly:
```bash
wasm-pack build --target web

```


2. Serve the project using a local web server (e.g., Python's `http.server` or `live-server`):
```bash
python3 -m http.server

```


3. Open `index.html` in your browser.

## Usage

1. Enter a phrase in the search bar (e.g., "Web Assembly").
2. Click **Solve**.
3. The application will process the phrase and display a list of found anagram combinations ranked by quality.
