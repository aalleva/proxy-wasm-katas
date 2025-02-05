use std::collections::HashMap;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(HttpHeadersJsonLoggerRoot)
    });
}}

struct HttpHeadersJsonLoggerRoot;

impl Context for HttpHeadersJsonLoggerRoot {}

impl RootContext for HttpHeadersJsonLoggerRoot {
    
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpHeadersJsonLoggerContext))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct HttpHeadersJsonLoggerContext;

impl Context for HttpHeadersJsonLoggerContext {}

impl HttpContext for HttpHeadersJsonLoggerContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        
        let mut headers_map: HashMap<String, String> = HashMap::new();
        for (header_name, header_value) in self.get_http_request_headers() {
            headers_map.insert(header_name.to_string(), header_value.to_string());
        }


        if let Ok(headers_json) = serde_json::to_string(&headers_map) {
            let _ = proxy_wasm::hostcalls::log(LogLevel::Info, &format!("Request headers {}:", headers_json));
        } else {
            let _ = proxy_wasm::hostcalls::log(LogLevel::Error, "Failed to serialize headers to JSON");
        }
        
        Action::Continue
    }
}