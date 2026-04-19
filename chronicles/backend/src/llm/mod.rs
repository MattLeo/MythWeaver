use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::models::GameState;
use crate::tools;

pub mod executor;

const MAX_TOOL_ITERATIONS: usize = 10;

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    base_url: String,
    model: String,
}

impl LlmClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            model,
        }
    }

    /// Run the full agentic loop: send messages, handle tool calls,
    /// repeat until a clean narrative response is returned.
    pub async fn run_agentic_loop(
        &self,
        pool: &SqlitePool,
        campaign_id: &str,
        system_prompt: &str,
        messages: Vec<ChatMessage>,
        game_state: &GameState,
    ) -> Result<AgentResult> {
        let available_tools = tools::tools_for_state(game_state);
        let mut current_messages = messages.clone();
        let mut tool_calls_made: Vec<ToolCallRecord> = vec![];
        let mut roll_request: Option<RollRequest> = None;

        for iteration in 0..MAX_TOOL_ITERATIONS {
            tracing::debug!("Agent loop iteration {}", iteration);

            let response = self.chat_completion(
                system_prompt,
                &current_messages,
                &available_tools,
            ).await?;

            let choice = response.choices.into_iter().next()
                .ok_or_else(|| anyhow::anyhow!("No choices in response"))?;

            let message = choice.message;

            // If no tool calls, we have a clean narrative response
            if message.tool_calls.is_none() || message.tool_calls.as_ref().map(|t| t.is_empty()).unwrap_or(true) {
                let content = message.content.unwrap_or_default();
                return Ok(AgentResult {
                    narrative: content,
                    tool_calls_made,
                    roll_request,
                });
            }

            // Process tool calls
            let tool_calls = message.tool_calls.unwrap();

            // Add assistant message with tool calls to history
            current_messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: message.content,
                tool_calls: Some(tool_calls.clone()),
                tool_call_id: None,
            });

            // Check if any tool call is request_roll (requires frontend interaction)
            for tc in &tool_calls {
                if tc.function.name == "request_roll" {
                    let args: Value = serde_json::from_str(&tc.function.arguments)?;
                    roll_request = Some(RollRequest {
                        tool_call_id: tc.id.clone(),
                        die: args["die"].as_str().unwrap_or("d20").to_string(),
                        skill: args["skill"].as_str().unwrap_or("").to_string(),
                        dc: args["dc"].as_i64().unwrap_or(10),
                        reason: args["reason"].as_str().unwrap_or("").to_string(),
                    });

                    // Return early — frontend must handle the roll
                    return Ok(AgentResult {
                        narrative: String::new(),
                        tool_calls_made,
                        roll_request,
                    });
                }
            }

            // Execute all tool calls
            for tc in &tool_calls {
                let args: Value = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(json!({}));

                tracing::info!("Executing tool: {} with args: {}", tc.function.name, args);

                let result = executor::execute_tool(
                    pool,
                    campaign_id,
                    &tc.function.name,
                    &args,
                ).await;

                let result_str = match result {
                    Ok(val) => serde_json::to_string_pretty(&val).unwrap_or_default(),
                    Err(e) => {
                        tracing::error!("Tool '{}' error: {}", tc.function.name, e);
                        format!("{{\"error\": \"{}\"}}", e)
                    }
                };

                tool_calls_made.push(ToolCallRecord {
                    tool_name: tc.function.name.clone(),
                    args: args.clone(),
                    result: result_str.clone(),
                });

                // Add tool result to message history
                current_messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: Some(result_str),
                    tool_calls: None,
                    tool_call_id: Some(tc.id.clone()),
                });
            }
        }

        Err(anyhow::anyhow!("Agent loop exceeded maximum iterations"))
    }

    async fn chat_completion(
        &self,
        system: &str,
        messages: &[ChatMessage],
        tools: &[Value],
    ) -> Result<ChatResponse> {
        let mut payload = json!({
            "model": self.model,
            "messages": build_messages(system, messages),
            "stream": false,
            "options": {
                "temperature": 0.85,
                "top_p": 0.95,
                "top_k": 20,
                "num_predict": 1024
            }
        });

        if !tools.is_empty() {
            payload["tools"] = json!(tools);
        }

        let url = format!("{}/v1/chat/completions", self.base_url);

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!("LLM API error {}: {}", status, body));
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response)
    }
}

fn build_messages(system: &str, messages: &[ChatMessage]) -> Vec<Value> {
    let mut result = vec![json!({
        "role": "system",
        "content": system
    })];

    for msg in messages {
        let mut m = json!({
            "role": msg.role
        });

        if let Some(content) = &msg.content {
            m["content"] = json!(content);
        }

        if let Some(tool_calls) = &msg.tool_calls {
            m["tool_calls"] = json!(tool_calls);
        }

        if let Some(tool_call_id) = &msg.tool_call_id {
            m["tool_call_id"] = json!(tool_call_id);
        }

        result.push(m);
    }

    result
}

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Serialize)]
pub struct AgentResult {
    pub narrative: String,
    pub tool_calls_made: Vec<ToolCallRecord>,
    pub roll_request: Option<RollRequest>,
}

#[derive(Debug, Serialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub args: Value,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollRequest {
    pub tool_call_id: String,
    pub die: String,
    pub skill: String,
    pub dc: i64,
    pub reason: String,
}