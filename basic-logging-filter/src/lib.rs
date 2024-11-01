use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(SimpleLoggingRoot)
    });
}}

struct SimpleLoggingRoot;

impl Context for SimpleLoggingRoot {}

impl RootContext for SimpleLoggingRoot {
    
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(SimpleLoggingHttpContext))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct SimpleLoggingHttpContext;

impl Context for SimpleLoggingHttpContext {}

impl HttpContext for SimpleLoggingHttpContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        let _ = proxy_wasm::hostcalls::log(LogLevel::Info, "Request received");
        Action::Continue
    }
}