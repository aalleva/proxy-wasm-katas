# Kata 4: Full HTTP Request-Response Logging as JSON

## Objective
Create a WebAssembly filter that captures both **HTTP request and response headers**, formats them as a **single structured JSON object**, and logs them in a single entry.

## Requirements
1. **Capture both request and response headers** when they are received.
2. **Structure the log entry as a single JSON object** containing:
   - Request headers
   - Response headers
   - Timestamp
   - Client IP (if available)
3. **Ensure efficient logging** without excessive performance overhead.
4. **Do not modify the request or response**—only observe and log.

## Steps

1. **Create a new struct to handle HTTP request-response logging**:
   - The struct should store request headers temporarily.

2. **Capture request headers when a request arrives**:
   - Extract all HTTP request headers.
   - Store them in the struct for later use.

3. **Capture response headers when the response is received**:
   - Extract all HTTP response headers.
   - Retrieve previously stored request headers.
   - Combine all data into a single JSON object.

4. **Format the final log as JSON**:
   - The log structure should look like this:
     ```json
     {
       "timestamp": "2025-02-05T12:34:56Z",
       "client_ip": "192.168.1.1",
       "request_headers": {
         "host": "example.com",
         "user-agent": "Mozilla/5.0"
       },
       "response_headers": {
         "content-type": "application/json",
         "server": "nginx"
       }
     }
     ```
   - Use `serde_json` for serialization.

5. **Compile & Test the Policy**:
   - Ensure the `.wasm` file is generated and behaves correctly.

## Expected Output
A **single log entry per request-response cycle** structured as a JSON object.

## Hints
<!-- Hint 1 -->
- Use `self.get_http_request_headers()` to retrieve request headers when they arrive.

<!-- Hint 2 -->
- Use `self.get_http_response_headers()` to retrieve response headers when they arrive.

<!-- Hint 3 -->
- Store request headers in a **struct field** so they can be used later when processing the response.

<!-- Hint 4 -->
- Consider adding a timestamp using `std::time::SystemTime` to make logs more useful.

## Next Steps
Once you've implemented the code, submit it for validation before moving to the next kata.
