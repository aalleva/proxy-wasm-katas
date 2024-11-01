# Kata 1: Basic Logging Filter

## Objective

Create a simple proxy filter in Rust that logs every incoming HTTP request. This exercise will help you set up your first proxy-wasm filter, focusing on handling HTTP request headers and logging events.

## Requirements

1. Set up a basic Rust project configured to compile to WebAssembly (WASM).
2. Implement a filter that:
   - Logs each incoming HTTP request header.
   - Allows the request to continue without any changes.

## Steps

1. **Project Setup**:
   - Create a new Rust library project named `basic-logging-filter`.
   - Configure the project to compile for the `wasm32-wasi` target.

2. **Logging Implementation**:
   - Write a filter that logs a message each time it receives HTTP request headers.
   - The log message should state that an HTTP request was received.

3. **Compile the Project**:
   - Ensure the project compiles to a `.wasm` file, which will be used as the filter.

## Expected Output

When this filter is running, you should see a log entry for each incoming HTTP request in the system where it's deployed (e.g., Envoy or Flex Gateway).