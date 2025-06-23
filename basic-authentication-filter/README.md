# Kata #7: Basic Authentication Policy

## Challenge

Create a Proxy-Wasm policy that implements [HTTP Basic Authentication](https://datatracker.ietf.org/doc/html/rfc7617).

The policy must:

- Read the `Authorization` header from the HTTP request.
- Validate credentials against a configured list of username/password pairs.
- Allow the request **only if** the credentials are valid.
- Otherwise, reject the request with:
  - HTTP `401 Unauthorized`
  - Header: `WWW-Authenticate: Basic realm="Restricted"`
  - A response body in **JSON** format describing the error (see below).

## Unauthorized Response Format

```
aalleva@aalleva-ltmwqj3 basic-authentication-filter % curl -v --location 'http://0.0.0.0:8081/httpbin/headers' \
--header 'Authorization: bWF4Om1heFRoZU11bGUz='
*   Trying 0.0.0.0:8081...
* Connected to 0.0.0.0 (0.0.0.0) port 8081
> GET /httpbin/headers HTTP/1.1
> Host: 0.0.0.0:8081
> User-Agent: curl/8.7.1
> Accept: */*
> Authorization: bWF4Om1heFRoZU11bGUz=
>
* Request completely sent off
< HTTP/1.1 401 Unauthorized
< content-type: application/json
< www-authenticate: Basic realm="Restricted"
< content-length: 89
< date: Mon, 23 Jun 2025 22:19:12 GMT
< server: Anypoint Flex Gateway
<
* Connection #0 to host 0.0.0.0 left intact
{"description":"Invalid credentials. Access denied.","error":"Unauthorized","status":401}%
```

## Hints

To implement this Kata you may want to look at the following crates:

- [`base64`](https://crates.io/crates/base64): for decoding the `Authorization` header value.
- [`serde`](https://crates.io/crates/serde): to deserialize the policy configuration.
