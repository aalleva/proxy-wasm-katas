# Kata 2: Logging HTTP Request Headers

## Objective

Extend the basic logging filter to log each HTTP request header received. This exercise builds on Kata 1 by implementing header inspection and logging. This will help you practice accessing and iterating over HTTP headers in a proxy-wasm filter.

## Requirements

1. Modify the `SimpleLoggingHttpContext` structure so that it logs each HTTP request header.
2. Log both the header name and the header value for each incoming request.
3. Ensure that the request continues without interruption after logging.

## Steps

1. **Logging HTTP Headers**:
   - Inside the `on_http_request_headers` function, access all incoming headers.
   - Iterate over each header and log its name and value.

2. **Compile the Project**:
   - Compile the project for the `wasm32-wasi` target to produce the updated `.wasm` file.

3. **Expected Output**:
   - When this filter is deployed, you should see each header of the HTTP request logged individually, showing both header names and values.
