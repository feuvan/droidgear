//! Model connectivity testing and diagnostics.
//!
//! Tests API connectivity for configured AI models with detailed diagnostics.
//! Supports two modes:
//! - **Ping**: lightweight HTTP probe on `/v1/models` (free, fast)
//! - **Inference**: real model call via reqwest (costs tokens, slower, but
//!   validates the model is actually usable)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::{Duration, Instant};

use crate::factory_settings::{self, Provider};

// ============================================================================
// Types
// ============================================================================

/// Test mode: ping (HTTP probe) or inference (real model call).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TestMode {
    Ping,
    Inference,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDiagnostics {
    pub success: bool,
    pub provider: String,
    pub model_id: String,
    pub latency_ms: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub timestamp: String,
    pub test_mode: TestMode,
    /// Actual model response text (inference mode only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_text: Option<String>,
    /// The prompt that was sent (inference mode only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelTestResult {
    pub model_id: String,
    pub model_name: String,
    pub diagnostics: ConnectionDiagnostics,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TestConfig {
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
    #[serde(default = "default_retry")]
    pub retry_on_failure: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_timeout() -> u32 {
    10
}
fn default_retry() -> bool {
    true
}
fn default_max_retries() -> u32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConnectivitySummary {
    pub total_models: u32,
    pub available_models: u32,
    pub unavailable_models: u32,
    pub avg_latency_ms: f64,
    pub last_updated: String,
}

// ============================================================================
// Model Tester
// ============================================================================

pub struct ModelTester {
    config: TestConfig,
    client: reqwest::Client,
}

impl ModelTester {
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_config(config: TestConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    /// Test connectivity by hitting a lightweight endpoint (e.g. /v1/models).
    /// This avoids inference costs while verifying API key and network.
    pub async fn test_model(
        &self,
        provider: &Provider,
        base_url: &str,
        api_key: &str,
        model_id: &str,
    ) -> ConnectionDiagnostics {
        let start = Instant::now();
        let provider_str = provider_to_string(provider);
        log::info!("Connectivity: ping test for model={model_id} provider={provider_str} base_url={base_url}");

        if api_key.is_empty() {
            return ConnectionDiagnostics {
                success: false,
                provider: provider_str,
                model_id: model_id.to_string(),
                latency_ms: start.elapsed().as_millis() as u32,
                error: Some("No API key configured".to_string()),
                timestamp: Utc::now().to_rfc3339(),
                test_mode: TestMode::Ping,
                response_text: None,
                prompt_used: None,
            };
        }

        let result = self.probe_endpoint(provider, base_url, api_key).await;
        let latency_ms = start.elapsed().as_millis() as u32;

        match result {
            Ok(()) => ConnectionDiagnostics {
                success: true,
                provider: provider_str,
                model_id: model_id.to_string(),
                latency_ms,
                error: None,
                timestamp: Utc::now().to_rfc3339(),
                test_mode: TestMode::Ping,
                response_text: None,
                prompt_used: None,
            },
            Err(e) => ConnectionDiagnostics {
                success: false,
                provider: provider_str,
                model_id: model_id.to_string(),
                latency_ms,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
                test_mode: TestMode::Ping,
                response_text: None,
                prompt_used: None,
            },
        }
    }

    /// Test connectivity with explicit provider string (for direct API testing).
    pub async fn test_model_direct(
        &self,
        provider: &str,
        base_url: &str,
        api_key: &str,
        model_id: &str,
    ) -> ConnectionDiagnostics {
        let start = Instant::now();
        log::info!("Connectivity: direct ping test for model={model_id} provider={provider} base_url={base_url}");

        if api_key.is_empty() {
            return ConnectionDiagnostics {
                success: false,
                provider: provider.to_string(),
                model_id: model_id.to_string(),
                latency_ms: start.elapsed().as_millis() as u32,
                error: Some("No API key configured".to_string()),
                timestamp: Utc::now().to_rfc3339(),
                test_mode: TestMode::Ping,
                response_text: None,
                prompt_used: None,
            };
        }

        let p = string_to_provider(provider);
        let result = self.probe_endpoint(&p, base_url, api_key).await;
        let latency_ms = start.elapsed().as_millis() as u32;

        match result {
            Ok(()) => ConnectionDiagnostics {
                success: true,
                provider: provider.to_string(),
                model_id: model_id.to_string(),
                latency_ms,
                error: None,
                timestamp: Utc::now().to_rfc3339(),
                test_mode: TestMode::Ping,
                response_text: None,
                prompt_used: None,
            },
            Err(e) => ConnectionDiagnostics {
                success: false,
                provider: provider.to_string(),
                model_id: model_id.to_string(),
                latency_ms,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
                test_mode: TestMode::Ping,
                response_text: None,
                prompt_used: None,
            },
        }
    }

    /// Test model by sending a real inference request via aisdk.
    /// This validates the model is actually usable (key valid, model exists, quota OK).
    pub async fn test_model_inference(
        &self,
        provider: &Provider,
        base_url: &str,
        api_key: &str,
        model_id: &str,
        prompt: &str,
    ) -> ConnectionDiagnostics {
        let start = Instant::now();
        let provider_str = provider_to_string(provider);
        log::info!("Connectivity: inference test for model={model_id} provider={provider_str} base_url={base_url}");
        log::debug!("Connectivity: inference prompt={prompt:?}");

        if api_key.is_empty() {
            return ConnectionDiagnostics {
                success: false,
                provider: provider_str,
                model_id: model_id.to_string(),
                latency_ms: start.elapsed().as_millis() as u32,
                error: Some("No API key configured".to_string()),
                timestamp: Utc::now().to_rfc3339(),
                test_mode: TestMode::Inference,
                response_text: None,
                prompt_used: Some(prompt.to_string()),
            };
        }

        let result = self
            .run_inference(provider, base_url, api_key, model_id, prompt)
            .await;
        let latency_ms = start.elapsed().as_millis() as u32;

        match result {
            Ok(text) => ConnectionDiagnostics {
                success: true,
                provider: provider_str,
                model_id: model_id.to_string(),
                latency_ms,
                error: None,
                timestamp: Utc::now().to_rfc3339(),
                test_mode: TestMode::Inference,
                response_text: Some(text),
                prompt_used: Some(prompt.to_string()),
            },
            Err(e) => ConnectionDiagnostics {
                success: false,
                provider: provider_str,
                model_id: model_id.to_string(),
                latency_ms,
                error: Some(e),
                timestamp: Utc::now().to_rfc3339(),
                test_mode: TestMode::Inference,
                response_text: None,
                prompt_used: Some(prompt.to_string()),
            },
        }
    }

    async fn probe_endpoint(
        &self,
        provider: &Provider,
        base_url: &str,
        api_key: &str,
    ) -> Result<(), String> {
        let base = base_url.trim_end_matches('/');
        let url = format!("{base}/v1/models");
        log::debug!("Connectivity: probe_endpoint url={url} provider={provider:?}");

        let mut req = self.client.get(&url);

        match provider {
            Provider::Anthropic => {
                req = req
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01");
            }
            _ => {
                req = req.header("Authorization", format!("Bearer {api_key}"));
            }
        }

        let resp = tokio::time::timeout(
            Duration::from_secs(self.config.timeout_seconds as u64),
            req.send(),
        )
        .await
        .map_err(|_| "Request timed out".to_string())?
        .map_err(|e| format!("Network error: {e}"))?;

        let status = resp.status();
        if status.is_success() {
            log::info!("Connectivity: probe_endpoint OK, url={url} status={status}");
            Ok(())
        } else {
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            let truncated = if body.len() > 500 {
                format!("{}...", &body[..500])
            } else {
                body
            };
            log::warn!(
                "Connectivity: probe_endpoint failed, url={url} status={status} body={truncated}"
            );
            Err(format!("HTTP {status}: {truncated}"))
        }
    }

    /// Run a real inference request via reqwest and return the response text.
    async fn run_inference(
        &self,
        provider: &Provider,
        base_url: &str,
        api_key: &str,
        model_id: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let base = base_url.trim_end_matches('/');
        let timeout_secs = self.config.timeout_seconds.max(30) as u64;
        log::debug!(
            "Connectivity: run_inference model={model_id} base_url={base} timeout={timeout_secs}s"
        );

        let (url, req) = match provider {
            Provider::Anthropic => {
                let url = format!("{base}/v1/messages");
                let body = serde_json::json!({
                    "model": model_id,
                    "max_tokens": 32,
                    "messages": [{"role": "user", "content": prompt}]
                });
                let r = self
                    .client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .json(&body);
                (url, r)
            }
            _ => {
                let url = format!("{base}/v1/chat/completions");
                let body = serde_json::json!({
                    "model": model_id,
                    "max_tokens": 32,
                    "messages": [{"role": "user", "content": prompt}]
                });
                let r = self
                    .client
                    .post(&url)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .json(&body);
                (url, r)
            }
        };

        log::debug!("Connectivity: inference POST {url}");

        let resp = tokio::time::timeout(Duration::from_secs(timeout_secs), req.send())
            .await
            .map_err(|_| "Inference request timed out".to_string())?
            .map_err(|e| format!("Inference network error: {e}"))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read inference response body: {e}"))?;

        if !status.is_success() {
            let truncated = if body.len() > 500 {
                format!("{}...", &body[..500])
            } else {
                body
            };
            log::warn!(
                "Connectivity: inference failed, url={url} status={status} body={truncated}"
            );
            return Err(format!("Inference HTTP {status}: {truncated}"));
        }

        let data: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            let truncated = if body.len() > 500 {
                format!("{}...", &body[..500])
            } else {
                body.clone()
            };
            log::warn!(
                "Connectivity: failed to parse inference response, url={url} body={truncated}"
            );
            format!("Failed to parse inference response: {e}")
        })?;

        // Extract response text based on provider format
        let text = match provider {
            Provider::Anthropic => data
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|block| block.get("text"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string(),
            _ => data
                .get("choices")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string(),
        };

        log::info!(
            "Connectivity: inference OK for model={model_id}, response_len={}",
            text.len()
        );

        // Truncate very long responses
        let truncated = if text.len() > 200 {
            format!("{}…", &text[..200])
        } else {
            text
        };

        Ok(truncated)
    }
}

impl Default for ModelTester {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: default_timeout(),
            retry_on_failure: default_retry(),
            max_retries: default_max_retries(),
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn provider_to_string(provider: &Provider) -> String {
    match provider {
        Provider::Anthropic => "anthropic".to_string(),
        Provider::Openai => "openai".to_string(),
        Provider::GenericChatCompletionApi => "generic".to_string(),
    }
}

fn string_to_provider(s: &str) -> Provider {
    match s {
        "anthropic" => Provider::Anthropic,
        "openai" => Provider::Openai,
        _ => Provider::GenericChatCompletionApi,
    }
}

// ============================================================================
// Public API
// ============================================================================

const DEFAULT_INFERENCE_PROMPT: &str = "Hi";

pub async fn test_all_model_connections() -> Result<Vec<ModelTestResult>, String> {
    test_all_model_connections_with_mode(TestMode::Ping, None).await
}

pub async fn test_all_model_connections_with_mode(
    mode: TestMode,
    prompt: Option<String>,
) -> Result<Vec<ModelTestResult>, String> {
    let tester = ModelTester::new();
    let mut results = Vec::new();
    let prompt_str = prompt.unwrap_or_else(|| DEFAULT_INFERENCE_PROMPT.to_string());

    let models = factory_settings::load_custom_models()?;
    for model in &models {
        let display = model
            .display_name
            .clone()
            .unwrap_or_else(|| model.model.clone());
        let id = model.id.clone().unwrap_or_else(|| model.model.clone());

        let diag = match mode {
            TestMode::Ping => {
                tester
                    .test_model(
                        &model.provider,
                        &model.base_url,
                        &model.api_key,
                        &model.model,
                    )
                    .await
            }
            TestMode::Inference => {
                tester
                    .test_model_inference(
                        &model.provider,
                        &model.base_url,
                        &model.api_key,
                        &model.model,
                        &prompt_str,
                    )
                    .await
            }
        };
        let available = diag.success;
        results.push(ModelTestResult {
            model_id: id,
            model_name: display,
            diagnostics: diag,
            is_available: available,
        });
    }

    Ok(results)
}

pub async fn test_specific_model_connection(model_id: &str) -> Result<ModelTestResult, String> {
    test_specific_model_connection_with_mode(model_id, TestMode::Ping, None).await
}

pub async fn test_specific_model_connection_with_mode(
    model_id: &str,
    mode: TestMode,
    prompt: Option<String>,
) -> Result<ModelTestResult, String> {
    let models = factory_settings::load_custom_models()?;
    let model = models
        .iter()
        .find(|m| m.id.as_deref() == Some(model_id))
        .ok_or_else(|| format!("Model {model_id} not found"))?;

    let tester = ModelTester::new();
    let display = model
        .display_name
        .clone()
        .unwrap_or_else(|| model.model.clone());
    let prompt_str = prompt.unwrap_or_else(|| DEFAULT_INFERENCE_PROMPT.to_string());

    let diag = match mode {
        TestMode::Ping => {
            tester
                .test_model(
                    &model.provider,
                    &model.base_url,
                    &model.api_key,
                    &model.model,
                )
                .await
        }
        TestMode::Inference => {
            tester
                .test_model_inference(
                    &model.provider,
                    &model.base_url,
                    &model.api_key,
                    &model.model,
                    &prompt_str,
                )
                .await
        }
    };
    let available = diag.success;

    Ok(ModelTestResult {
        model_id: model_id.to_string(),
        model_name: display,
        diagnostics: diag,
        is_available: available,
    })
}

pub fn get_connectivity_summary(results: &[ModelTestResult]) -> ConnectivitySummary {
    let total = results.len();
    let available = results.iter().filter(|r| r.is_available).count();
    let unavailable = total - available;

    let avg_latency = if available > 0 {
        results
            .iter()
            .filter(|r| r.diagnostics.success)
            .map(|r| r.diagnostics.latency_ms as u64)
            .sum::<u64>() as f64
            / available as f64
    } else {
        0.0
    };

    ConnectivitySummary {
        total_models: total as u32,
        available_models: available as u32,
        unavailable_models: unavailable as u32,
        avg_latency_ms: avg_latency,
        last_updated: Utc::now().to_rfc3339(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testconfig_default() {
        let config = TestConfig::default();
        assert_eq!(config.timeout_seconds, 10);
        assert!(config.retry_on_failure);
        assert_eq!(config.max_retries, 2);
    }

    #[test]
    fn test_connectivity_summary() {
        let results = vec![
            ModelTestResult {
                model_id: "model1".to_string(),
                model_name: "Model 1".to_string(),
                diagnostics: ConnectionDiagnostics {
                    success: true,
                    provider: "openai".to_string(),
                    model_id: "model1".to_string(),
                    latency_ms: 100,
                    error: None,
                    timestamp: Utc::now().to_rfc3339(),
                    test_mode: TestMode::Ping,
                    response_text: None,
                    prompt_used: None,
                },
                is_available: true,
            },
            ModelTestResult {
                model_id: "model2".to_string(),
                model_name: "Model 2".to_string(),
                diagnostics: ConnectionDiagnostics {
                    success: false,
                    provider: "anthropic".to_string(),
                    model_id: "model2".to_string(),
                    latency_ms: 0,
                    error: Some("Failed".to_string()),
                    timestamp: Utc::now().to_rfc3339(),
                    test_mode: TestMode::Ping,
                    response_text: None,
                    prompt_used: None,
                },
                is_available: false,
            },
        ];

        let summary = get_connectivity_summary(&results);
        assert_eq!(summary.total_models, 2);
        assert_eq!(summary.available_models, 1);
        assert_eq!(summary.unavailable_models, 1);
        assert_eq!(summary.avg_latency_ms, 100.0);
    }

    #[test]
    fn test_empty_summary() {
        let results: Vec<ModelTestResult> = vec![];
        let summary = get_connectivity_summary(&results);
        assert_eq!(summary.total_models, 0);
        assert_eq!(summary.avg_latency_ms, 0.0);
    }

    #[test]
    fn test_provider_string_conversion() {
        assert_eq!(provider_to_string(&Provider::Anthropic), "anthropic");
        assert_eq!(provider_to_string(&Provider::Openai), "openai");
        assert_eq!(
            provider_to_string(&Provider::GenericChatCompletionApi),
            "generic"
        );
    }
}
