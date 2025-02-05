# Kata 3: Log HTTP Headers as JSON

## Objective

Modify the filter to log all HTTP request headers in a single JSON-formatted entry. This will make the logs easier to read and analyze.

## Requirements

1. Retrieve all HTTP request headers.
2. Format the headers as a single JSON object where each header name is a key, and its value is the corresponding value.
3. Log the JSON object as a single entry in the log.

## Steps

1. **Convert Headers to JSON**:
   - Retrieve all HTTP headers and convert them into a JSON object.
   - You can use a Rust library like `serde_json` to help with JSON serialization.

2. **Log the JSON**:
   - Log the JSON object as a single log entry.

3. **Compile the Project**:
   - Compile the project to produce the updated `.wasm` file.

4. **Expected Output**:
   - When this filter is deployed, you should see a single log entry with all headers formatted as JSON, rather than individual log entries per header.

## Hints
<!-- Hint 1 -->
- Use `self.get_http_request_headers()` to retrieve all headers as a vector of `(header_name, header_value)` pairs.

<!-- Hint 2 -->
- Create a `HashMap` and insert each header name and value pair into it.

<!-- Hint 3 -->
- Use `serde_json` to serialize the `HashMap` to a JSON string.

## Dependencies
- Add `serde` and `serde_json` to your `Cargo.toml` for JSON serialization.
