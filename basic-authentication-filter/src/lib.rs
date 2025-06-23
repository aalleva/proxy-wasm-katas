use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::Deserialize;
use base64::prelude::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(BasicAuthenticationRoot {
            config: BasicAuthenticationConfig::default()
        })
    });
}}

#[derive(Deserialize, Clone, Default, Debug)]
struct BasicAuthenticationConfig {
    username: String,
    password: String,
}
struct BasicAuthenticationRoot {
    config: BasicAuthenticationConfig,
}

impl Context for BasicAuthenticationRoot {}

impl RootContext for BasicAuthenticationRoot {
    
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(BasicAuthenticationHttpContext {
            config: self.config.clone(),
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

        true
    }
}

struct BasicAuthenticationHttpContext {
    config: BasicAuthenticationConfig,
}

impl Context for BasicAuthenticationHttpContext {}

impl HttpContext for BasicAuthenticationHttpContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        
        if let Some(authorization_header) = self.get_http_request_header("authorization") {
            
            if let Some (encoded) = authorization_header.strip_prefix("Basic ") {

                match BASE64_STANDARD.decode(encoded) {
                    Ok(decoded_bytes) => {
                        if let Ok(credentials) = String::from_utf8(decoded_bytes) {
                            if let Some((username, password)) = credentials.split_once(':') {
                                if username == self.config.username && password == self.config.password {
                                    proxy_wasm::hostcalls::log(LogLevel::Info, "Basic auth success").ok();
                                    return Action::Continue;
                                }
                            }
                        }
                    },
                    Err(_) => {
                        proxy_wasm::hostcalls::log(LogLevel::Warn, "Invalid base64 encoding in 'Authorization' header").ok();
                    }
                }

            } else {
                proxy_wasm::hostcalls::log(LogLevel::Warn, "Invalid 'Authorization' header").ok();
            }

        } else {
            proxy_wasm::hostcalls::log(LogLevel::Warn, "No 'Authorization' header found").ok();
        }

        let json_payload = serde_json::json!({
            "error": "Unauthorized",
            "description": "Invalid credentials. Access denied.",
            "status": 401
        }).to_string();

        proxy_wasm::hostcalls::send_http_response(
            401, 
            vec![
                ("Content-Type", "application/json"),
                ("WWW-Authenticate", r#"Basic realm="Restricted""#)
            ],
            Some(json_payload.as_bytes())
        ).ok();

        Action::Pause

    }
}