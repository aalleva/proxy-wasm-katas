use std::time::SystemTime;
use std::vec;

use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::Deserialize;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(RateLimitRoot{
            config: RateLimitConfig::default()
        })
    });
}}

#[derive(Deserialize, Clone, Default, Debug)]
struct RateLimitConfig {
    max_requests: u64,
    ttl_seconds: u64,
}

struct RateLimitRoot {
    config: RateLimitConfig,
}

impl Context for RateLimitRoot {}

impl RootContext for RateLimitRoot {
    
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(RateLimitFilter {
            config: self.config.clone()
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn on_configure(&mut self, _plugin_configuration_size: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            self.config = serde_json::from_slice(config_bytes.as_slice()).unwrap_or_default();
            proxy_wasm::hostcalls::log(LogLevel::Info, &format!("config: {:?}", self.config)).ok();
        }

        true // use defaults
    }
}

struct RateLimitFilter {
    config: RateLimitConfig,
}

impl Context for RateLimitFilter {}

impl HttpContext for RateLimitFilter {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        
        // Extract 'client-id' from headers
        if let Some(client_id) = self.get_http_request_header("client-id") {

            // Get the current timestamp
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let reset_key = format!("rate_limit_reset_{}", client_id);
            let counter_key = format!("rate_limit_count_{}", client_id);

            // Check last reset timestamp
            let (ts_bytes_opt, _) = proxy_wasm::hostcalls::get_shared_data(&reset_key).unwrap_or((None, Some(0)));
            let last_reset = ts_bytes_opt
                .and_then(|d| Some(u64::from_be_bytes(d.try_into().unwrap_or([0; 8]))))
                .unwrap_or(0);

            // Reset counter if more than 60 seconds have passed
            if now.saturating_sub(last_reset) > self.config.ttl_seconds {
                proxy_wasm::hostcalls::set_shared_data(&reset_key, Some(&now.to_be_bytes()), None).ok();
                proxy_wasm::hostcalls::set_shared_data(&counter_key, Some(&0_u64.to_be_bytes()), None).ok();
                proxy_wasm::hostcalls::log(LogLevel::Info, &format!("Reset rate limit ")).ok();
            }
            
            // Retrieve the current request count for the Client-ID
            let (data, cas) = proxy_wasm::hostcalls::get_shared_data(&&counter_key)
                .unwrap_or((None, Some(0)));
            let mut count = data
                .and_then(|d| Some(u64::from_be_bytes(d.try_into().unwrap_or([0; 8]))))
                .unwrap_or(0);
            
            // Increment the request count
            count += 1;
            proxy_wasm::hostcalls::set_shared_data(
                    &counter_key, 
                    Some(&count.to_be_bytes()), 
                    cas
            ).ok();

            // Check if the client exceeded the rate limit
            if count > self.config.max_requests {
                let log_msg = format!("Client {} exceeded rate limit ({} reqs). Blocking request.", client_id, count);
                proxy_wasm::hostcalls::log(LogLevel::Warn, &log_msg).ok();

                // Send 429 Too Many Requests Response
                proxy_wasm::hostcalls::send_http_response(
                    429,
                    vec![("Content-Type", "text/plain"), ("Retry-After", "60")],
                    Some(b"Too Many Requests"),
                ).ok();
                
                return Action::Pause;
            }            

            let log_msg = format!("Client {} made a request. Allowing.", client_id);
            proxy_wasm::hostcalls::log(LogLevel::Info, &log_msg).ok();        


        } else {
            proxy_wasm::hostcalls::log(LogLevel::Info, "Client-ID header missing. Allowing request.").ok();
        }
        
        Action::Continue
    }
}