use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::time::SystemTime;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

fn get_current_timestamp() -> String {
    let system_time = SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();
    datetime.to_rfc3339()
}

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(HttpFullRequestResponseLoggerRoot)
    });
}}

struct HttpFullRequestResponseLoggerRoot;

impl Context for HttpFullRequestResponseLoggerRoot {}

impl RootContext for HttpFullRequestResponseLoggerRoot {
    
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpFullRequestResponseLoggerContext {
            request_headers: None,
            response_headers: None,
            timestamp: get_current_timestamp(),
            client_ip: "unknown".to_string(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct HttpFullRequestResponseLoggerContext {
    request_headers: Option<HashMap<String, String>>,
    response_headers: Option<HashMap<String, String>>,
    timestamp: String,
    client_ip: String,
}

impl Context for HttpFullRequestResponseLoggerContext {}

impl HttpContext for HttpFullRequestResponseLoggerContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        
        // Capture the current timestamp
        self.timestamp = get_current_timestamp();

        // Retrieve and store request headers
        let headers = self.get_http_request_headers();
        if headers.is_empty() {
            proxy_wasm::hostcalls::log(LogLevel::Error, "Failed to retrieve request headers.").ok();
        } else {
            self.request_headers = Some(headers.into_iter().collect());
        }
        
        // Attempt to retrieve the client IP from the "x-forwarded-for" header
        self.client_ip = self.get_http_request_header("x-forwarded-for")
            .or_else(|| {
                self.get_property(vec!["downstream", "remote_address"])
                    .and_then(|addr| std::str::from_utf8(&addr).ok().map(|s| s.to_string()))
            })
            .unwrap_or_else(|| "unknown".to_string());

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        
        // Update timestamp to ensure accurate response logging
        self.timestamp = get_current_timestamp();

        // Retrieve and store response headers
        let headers = self.get_http_response_headers();
        if headers.is_empty() {
            proxy_wasm::hostcalls::log(LogLevel::Error, "Failed to retrieve response headers.").ok();
        } else {
            self.response_headers = Some(headers.into_iter().collect());
        }

        // Log the collected information as JSON
        if let (Some(request_headers), Some(response_headers)) = (
            &self.request_headers,
            &self.response_headers
        ) {
            let log_entry = serde_json::json!({
                "timestamp": self.timestamp,
                "client_ip": self.client_ip,
                "request_headers": request_headers,
                "response_headers": response_headers,
            });

            match serde_json::to_string(&log_entry) {
                Ok(log_json) => {
                    let log_message = format!("[Proxy-Wasm] HTTP Log: {}", log_json);
                    proxy_wasm::hostcalls::log(LogLevel::Info, &log_message).ok();
                },
                Err(err) => {
                    proxy_wasm::hostcalls::log(LogLevel::Error, &format!("Failed to serialize log entry: {:?}", err)).ok();
                }
            }
        }

        Action::Continue
    }

}