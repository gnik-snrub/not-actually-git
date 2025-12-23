# NAG - Not Actually Git

A Git-like version control system written from scratch in Rust. This educational project re-implements Git's core architecture to understand how distributed version control works internally, from object storage and branching to merge conflict resolution.

## Overview

NAG (Not Actually Git) is a Git-like version control system written from scratch in Rust. This educational project re-implements Git's core architecture to understand how distributed version control works internally, from object storage and branching to merge conflict resolution.

## Features

NAG implements essential Git operations including repository initialization, file staging, commits, branching, merging, and remote management. The system supports three-way merges with conflict resolution and provides colored status output for an intuitive user experience.

## Installation

Build NAG from source using Rust's cargo package manager. Clone the repository and run `cargo build --release` to compile the binary, which will be available in `target/release/nag`. No external dependencies beyond the Rust toolchain are required.

## Quick Start

Create a new repository with `nag init`, add files using `nag add <filename>`, commit changes with `nag commit "message"`, and check your working directory status using `nag status`. The workflow mirrors Git's familiar commands, making it easy to transition between systems.

## Commands

NAG provides a comprehensive command set: `init`, `status`, `add`, `commit`, `branch`, `checkout`, `merge`, `tag`, `restore`, `resolve`, and `remote`. Each command follows Git's conventions while implementing the underlying operations using NAG's custom object storage and reference system. Remote functionality includes `add`, `remove`, and `fetch` operations only.

## Architecture

The project is divided into core modules (hash, index, tree, refs, diff) and command handlers. Objects are stored in a content-addressable system using `.nag/objects/`, the index tracks file states and conflicts, and references manage branches and tags in a Git-like hierarchy.

## Testing

Run the comprehensive test suite with `cargo test`. The project includes unit tests for each core module and integration tests for command workflows, using temporary repositories to ensure isolated test environments and thorough coverage of all functionality.

## Limitations

NAG differs from Git in several ways: it supports only local remotes with `add`, `remove`, and `fetch` operations (no push functionality), provides basic conflict resolution without advanced merge tools, and is designed for single-user workflows rather than collaborative development.