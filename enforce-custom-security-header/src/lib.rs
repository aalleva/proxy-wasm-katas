use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(FilterRoot)
    });
}}

struct FilterRoot;

impl Context for FilterRoot {}

impl RootContext for FilterRoot {
    
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(FilterContext))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct FilterContext;

impl Context for FilterContext {}

impl HttpContext for FilterContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        
        if let Some(header_value) = self.get_http_request_header("Authorization") {
            let log_message = format!("Injecting X-Internal-Auth: {}", header_value);
            proxy_wasm::hostcalls::log(LogLevel::Info, &log_message).ok();
            self.set_http_request_header("X-Internal-Auth", Some(&header_value));
        } else {
            let _ = proxy_wasm::hostcalls::log(LogLevel::Warn, "Authorization header not found.");
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        
        if self.get_http_response_header("X-Frame-Options").is_none() {
            self.set_http_response_header("X-Frame-Options", Some("DENY"));
            proxy_wasm::hostcalls::log(LogLevel::Info, "Injected X-Frame-Options: DENY").ok();
        } else {
            proxy_wasm::hostcalls::log(LogLevel::Info, "X-Frame-Options header already exists. No modification needed.").ok();
        }

        Action::Continue
    }

}