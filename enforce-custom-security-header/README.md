# Kata #5: Enforcing a Custom Security Header

## Objective
In a microservices environment, APIs should enforce security headers to protect against attacks and ensure compliance with internal policies.

This Proxy-Wasm filter will:
- Automatically inject a security-related header into all incoming requests.
- Add a security header to all outgoing responses.
- Log modifications to ensure proper validation.

## Requirements
1. Modify request headers:
   - If the `Authorization` header is present, copy its value into a custom header `X-Internal-Auth`.
   - If missing, log a warning.

2. Modify response headers:
   - Ensure `X-Frame-Options: DENY` is always added to responses.

3. Log all modified headers:
   - Log any header modifications for debugging purposes.

## Steps
1. Implement the Proxy-Wasm filter to modify request and response headers.
2. Use `set_http_request_header` to inject `X-Internal-Auth` into requests.
3. Use `set_http_response_header` to inject `X-Frame-Options` into responses.
4. Log all modifications for verification.
5. Test the filter by making requests and verifying the injected headers.

## Expected Output
- The `X-Internal-Auth` header should be added to requests if `Authorization` is present.
- The `X-Frame-Options: DENY` header should be present in all responses.
- Logs should display the modifications made to request and response headers.

## Dependencies
- Rust with `wasm32-wasi` target installed.
- Proxy-Wasm Rust SDK.
- `chrono` crate for timestamp handling.
