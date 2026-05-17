use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::models::GameState;
use crate::tools;

pub mod executor;
pub mod prompt;

const MAX_TOOL_ITERATIONS: usize = 10;
const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";
const GEMINI_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions";

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    api_key: String,
    model: String,
}

impl LlmClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }


    pub async fn run_agentic_loop(...) -> Result<AgentResult> {
    let available_tools = tools::tools_for_state(game_state);
    let anthropic_tools = convert_tools_to_anthropic(&available_tools);
    let mut current_messages = messages.clone();
    let mut tool_calls_made: Vec<ToolCallRecord> = vec![];
    let mut roll_request: Option<RollRequest> = None;

    for iteration in 0..MAX_TOOL_ITERATIONS {
        tracing::debug!("Agent loop iteration {}", iteration);

        let response = self.chat_completion_anthropic(
            system_prompt,
            &current_messages,
            &anthropic_tools,
        ).await?;

        let stop_reason = response.stop_reason.as_deref().unwrap_or("end_turn");

        // Extract text and tool_use blocks
        let narrative_text: String = response.content.iter()
            .filter(|b| b.block_type == "text")
            .filter_map(|b| b.text.clone())
            .collect::<Vec<_>>()
            .join("");

        let tool_uses: Vec<AnthropicToolUse> = response.content.iter()
            .filter(|b| b.block_type == "tool_use")
            .filter_map(|b| Some(AnthropicToolUse {
                id: b.id.clone()?,
                name: b.name.clone()?,
                input: b.input.clone()?,
            }))
            .collect();

        if stop_reason == "end_turn" || tool_uses.is_empty() {
            return Ok(AgentResult {
                narrative: narrative_text,
                tool_calls_made,
                roll_request,
            });
        }

        // Store assistant turn (text + tool_use blocks)
        current_messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: if narrative_text.is_empty() { None } else { Some(narrative_text.clone()) },
            anthropic_content: Some(response.content.clone()),
            tool_results: None,
        });

        // Execute tools and collect results
        let mut tool_result_blocks: Vec<Value> = vec![];
        for tc in &tool_uses {
            let input = tc.input.clone();
            tracing::info!("Executing tool: {} with args: {}", tc.name, input);

            // Handle roll requests
            if tc.name == "request_roll" {
                let die = input["die"].as_str().unwrap_or("d20").to_string();
                let skill = input["skill"].as_str().unwrap_or("").to_string();
                let dc = input["dc"].as_i64().unwrap_or(10);
                let reason = input["reason"].as_str().unwrap_or("").to_string();
                roll_request = Some(RollRequest {
                    tool_call_id: tc.id.clone(),
                    die, skill, dc, reason,
                });
                tool_result_blocks.push(json!({
                    "type": "tool_result",
                    "tool_use_id": tc.id,
                    "content": "Roll requested. Waiting for player."
                }));
                continue;
            }

            let result = executor::execute_tool(pool, campaign_id, &tc.name, &input).await;
            let result_str = match result {
                Ok(val) => serde_json::to_string_pretty(&val).unwrap_or_default(),
                Err(e) => { tracing::error!("Tool '{}' error: {}", tc.name, e); format!("{{\"error\": \"{}\"}}", e) }
            };

            tool_calls_made.push(ToolCallRecord {
                tool_name: tc.name.clone(),
                args: input,
                result: result_str.clone(),
            });

            tool_result_blocks.push(json!({
                "type": "tool_result",
                "tool_use_id": tc.id,
                "content": result_str
            }));
        }

        current_messages.push(ChatMessage {
            role: "user".to_string(),
            content: None,
            anthropic_content: None,
            tool_results: Some(tool_result_blocks),
        });
    }

    Err(anyhow::anyhow!("Agent loop exceeded maximum iterations"))
}


    pub async fn narrate_combat_result(&self, system: &str, result: &Value) -> Result<String> {
        let prompt = format!(
            "Combat result: {}. Write exactly 1-2 sentences of vivid combat narrative describing this outcome. Use ONLY the names and details provided in the combat result above. Do not invent or add any characters, enemies, or details not present in the result. No markdown. No lists.",
            serde_json::to_string(result).unwrap_or_default()
        );
        let messages = vec![ChatMessage::user(&prompt)];
        let response = self.chat_completion(system, &messages, &[]).await?;
        let content = response.choices.into_iter().next()
            .and_then(|c| c.message.content)
            .unwrap_or_else(|| "The attack lands.".to_string());
        Ok(content)
    }

    async fn chat_completion(
        &self,
        system: &str,
        messages: &[ChatMessage],
        tools: &[Value],
    ) -> Result<OpenRouterResponse> {
        let mut openrouter_messages: Vec<Value> = vec![];
 
        // System prompt goes as first message
        openrouter_messages.push(json!({
            "role": "system",
            "content": system
        }));
 
        // Build the rest of the messages
        for msg in messages {
            // Tool results — each becomes a separate "tool" role message
            if let Some(results) = &msg.tool_results {
                for result in results {
                    openrouter_messages.push(json!({
                        "role": "tool",
                        "tool_call_id": result["tool_use_id"],
                        "content": result["content"]
                    }));
                }
                continue;
            }
 
            // Assistant messages with tool calls stored
            if msg.role == "assistant" {
                if let Some(blocks) = &msg.anthropic_content {
                    let text = blocks.iter()
                        .find(|b| b.block_type == "text")
                        .and_then(|b| b.text.clone())
                        .unwrap_or_default();
 
                    let tool_calls: Vec<Value> = blocks.iter()
                        .filter(|b| b.block_type == "tool_use")
                        .filter_map(|b| {
                            let id = b.id.as_ref()?;
                            let name = b.name.as_ref()?;
                            let input = b.input.as_ref()?;
                            Some(json!({
                                "id": id,
                                "type": "function",
                                "function": {
                                    "name": name,
                                    "arguments": serde_json::to_string(input).unwrap_or_default()
                                }
                            }))
                        })
                        .collect();
 
                    let mut assistant_msg = json!({ "role": "assistant" });
                    if !text.is_empty() {
                        assistant_msg["content"] = json!(text);
                    }
                    if !tool_calls.is_empty() {
                        assistant_msg["tool_calls"] = json!(tool_calls);
                    }
                    openrouter_messages.push(assistant_msg);
                    continue;
                }
            }
 
            // Plain user or assistant message
            openrouter_messages.push(json!({
                "role": msg.role,
                "content": msg.content.as_deref().unwrap_or("")
            }));
        }
 
        let mut payload = json!({
            "model": self.model,
            "max_tokens": 1024,
            "messages": openrouter_messages,
        });
 
        if !tools.is_empty() {
            payload["tools"] = json!(tools);
        }
 
        let response = self.client
            .post(OPENROUTER_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://mythweaver.app")
            .header("X-Title", "MythWeaver Chronicles")
            .json(&payload)
            .send()
            .await?;
 
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!("OpenRouter API error {}: {}", status, body));
        }
 
        let or_response: OpenRouterResponse = response.json().await?;
        Ok(or_response)
    }
}

// ─── Tool format conversion ───────────────────────────────────────────────────

fn convert_tools_to_anthropic(tools: &[Value]) -> Vec<Value> {
    tools.iter().map(|t| {
        let function = &t["function"];
        json!({
            "name": function["name"],
            "description": function["description"],
            "input_schema": function["parameters"]
        })
    }).collect()
}

fn convert_tools_for_openrouter(tools: &[Value]) -> Vec<Value> {
    tools.iter().map(|t| {
        let function = &t["function"];
        json!({
            "type": "function",
            "function": {
                "name": function["name"],
                "description": function["description"],
                "parameters": function["parameters"]
            }
        })
    }).collect()
}

// ─── Message building ─────────────────────────────────────────────────────────

fn build_anthropic_messages(messages: &[ChatMessage]) -> Vec<Value> {
    messages.iter().map(|msg| {
        if let Some(results) = &msg.tool_results {
            return json!({
                "role": "user",
                "content": results
            });
        }

        if msg.role == "assistant" {
            if let Some(content_blocks) = &msg.anthropic_content {
                return json!({
                    "role": "assistant",
                    "content": content_blocks
                });
            }
        }

        json!({
            "role": msg.role,
            "content": msg.content.as_deref().unwrap_or("")
        })
    }).collect()
}

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    pub anthropic_content: Option<Vec<AnthropicContentBlock>>,
    pub tool_results: Option<Vec<Value>>,
}

impl ChatMessage {
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content.to_string()),
            anthropic_content: None,
            tool_results: None,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: Some(content.to_string()),
            anthropic_content: None,
            tool_results: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,
}

#[derive(Debug, Clone)]
struct AnthropicToolUse {
    pub id: String,
    pub name: String,
    pub input: Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    pub content: Vec<AnthropicContentBlock>,
    pub stop_reason: Option<String>,
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

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    pub choices: Vec<OpenRouterChoice>,
}
 
#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    pub message: OpenRouterMessage,
    pub finish_reason: Option<String>,
}
 
#[derive(Debug, Deserialize)]
struct OpenRouterMessage {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<OpenRouterToolCall>>,
}
 
#[derive(Debug, Deserialize)]
struct OpenRouterToolCall {
    pub id: String,
    pub function: OpenRouterFunction,
}
 
#[derive(Debug, Deserialize)]
struct OpenRouterFunction {
    pub name: String,
    pub arguments: String, // JSON string, needs parsing
}
