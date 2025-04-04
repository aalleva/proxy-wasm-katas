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
            config: self.config.clone(),
            remaining_quota: Some(self.config.max_requests),
            seconds_until_reset: Some(self.config.ttl_seconds),
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
    remaining_quota: Option<u64>,
    seconds_until_reset: Option<u64>,
}

fn increment_shared_counter(key: &str) -> Result<u64, ()> {
    for _ in 0..5 {
        let (data, cas_token) = proxy_wasm::hostcalls::get_shared_data(key)
            .unwrap_or((None, None));

        let mut current = data
            .map(|d| u64::from_be_bytes(d.try_into().unwrap_or([0; 8])))
            .unwrap_or(0);       
        
        current += 1;

        let result = proxy_wasm::hostcalls::set_shared_data(
            key, 
            Some(&current.to_be_bytes()), 
            cas_token
        );
        
        if result.is_ok() {
            return Ok(current);
        }

    }

    Err(())
}

fn reset_counter(reset_key: &str, counter_key: &str, ttl: u64, now: u64, client_id: &String) -> Option<u64> {

    for _ in 0..5 {
        // Check last reset timestamp
        let (data, cas_token) = proxy_wasm::hostcalls::get_shared_data(reset_key)
            .unwrap_or((None, None));
        let last_reset = data
            .map(|d| u64::from_be_bytes(d.try_into().unwrap_or([0; 8])))
            .unwrap_or(0);

        // Reset counter if more than ttl seconds have passed
        let elapsed = now.saturating_sub(last_reset);
        if elapsed > ttl {
            
            let result = proxy_wasm::hostcalls::set_shared_data(
                reset_key,
                Some(&now.to_be_bytes()),
                cas_token
            );

            if result.is_ok() {
                proxy_wasm::hostcalls::set_shared_data(counter_key, Some(&0_u64.to_be_bytes()), None).ok();
                proxy_wasm::hostcalls::log(LogLevel::Info, &format!("Reset rate limit for client {}", client_id)).ok();
                return Some(ttl);
            }
        } else {
            return Some(ttl.saturating_sub(elapsed));
        }
    }

    None
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

            // Reset counter or get seconds until reset
            self.seconds_until_reset = reset_counter(&reset_key, &counter_key, self.config.ttl_seconds, now, &client_id);
            
            // Increment the request count
            let count = match increment_shared_counter(&counter_key) {
                Ok(val) => val,
                Err(_) => {
                    proxy_wasm::hostcalls::log(LogLevel::Error, "CAS failed. Could not update request counter.").ok();
                    
                    // Send 500 Internal Server Error Response
                    let json_payload = serde_json::json!({
                        "error": "Internal Server Error",
                        "message": "Could not update request counter."
                    }).to_string();
                    
                    
                    proxy_wasm::hostcalls::send_http_response(
                        500,
                        vec![("Content-Type", "text/plain")],
                        Some(json_payload.as_bytes()),
                    ).ok();
                    
                    return Action::Pause;
                }
            };

            // Check if the client exceeded the rate limit
            if count > self.config.max_requests {
                let log_msg = format!("Client {} exceeded rate limit ({} reqs). Blocking request.", client_id, count);
                proxy_wasm::hostcalls::log(LogLevel::Warn, &log_msg).ok();

                // Send 429 Too Many Requests Response
                let json_payload = serde_json::json!({
                    "error": "Too Many Requests",
                    "message": format!("Client {} exceeded rate limit ({} reqs).", client_id, count),
                    "retry_after": self.seconds_until_reset
                }).to_string();
                
                proxy_wasm::hostcalls::send_http_response(
                    429,
                    vec![("Content-Type", "application/json")],
                    Some(json_payload.as_bytes()),
                ).ok();
                
                return Action::Pause;
            } else {
                let remaining = self.config.max_requests.saturating_sub(count);
                self.remaining_quota = Some(remaining);        
            }           

            let log_msg = format!("Client {} made a request. Allowing.", client_id);
            proxy_wasm::hostcalls::log(LogLevel::Info, &log_msg).ok();        


        } else {
            proxy_wasm::hostcalls::log(LogLevel::Info, "Client-ID header missing. Allowing request.").ok();
        }
        
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        
        self.set_http_response_header("X-RateLimit-Limit", Some(&self.config.max_requests.to_string()));
        
        if let Some(seconds_until_reset) = self.seconds_until_reset {
            self.set_http_response_header("X-RateLimit-Reset", Some(&seconds_until_reset.to_string()));
        }

        if let Some(remaining_quota) = self.remaining_quota {
            self.set_http_response_header("X-RateLimit-Remaining", Some(&remaining_quota.to_string()));
        
        }
        
        Action::Continue
    }

}