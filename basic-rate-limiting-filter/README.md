# Kata #6: Rate Limiting Based on Client ID

## Objective
This Proxy-Wasm filter enforces **rate limiting** by restricting the number of requests allowed per **Client ID**. Clients exceeding the limit will receive a **429 Too Many Requests** response.

## Requirements
1. **Identify Clients**
   - Extract the `Client-Id` header from incoming requests.
   - If `Client-Id` is missing, log a **warning** but allow the request.

2. **Enforce Rate Limits**
   - Allow **5 requests per minute** per `Client-Id`.
   - If a client exceeds the limit, respond with:
     ```
     HTTP/1.1 429 Too Many Requests
     Content-Type: text/plain
     Retry-After: 60
     ```

3. **Track Requests Using an In-Memory Counter**
   - Store request counts per `Client-Id`.
   - Reset the counter **every 60 seconds**.

4. **Log Exceeded Rate Limits**
   - If a request is blocked, log:
     ```
     Client <Client-Id> exceeded rate limit. Blocking request.
     ```

## Steps
1. Extract the `Client-Id` header from incoming requests.
2. Maintain a **per-client request counter** in memory.
3. Implement a **time-based reset mechanism** (every 60 seconds).
4. If a client exceeds **5 requests per minute**, return a `429 Too Many Requests`.
5. Log **every rate-limit violation**.

## Expected Output
- **Normal requests within the limit:**
  ```
  [Proxy-Wasm] Client abc123 made a request. Allowing.
  ```
- **When a client exceeds the limit:**
  ```
  [Proxy-Wasm] Client abc123 exceeded rate limit. Blocking request.
  ```

## Dependencies
- Rust with `wasm32-wasi` target installed.
- Proxy-Wasm Rust SDK.
- `std::collections::HashMap` for tracking requests.
- `proxy_wasm::hostcalls::send_http_response()` for blocking requests.
