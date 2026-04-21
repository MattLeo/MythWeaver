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
const ANTHROPIC_VERSION: &str = "2023-06-01";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";

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

    pub async fn run_agentic_loop(
        &self,
        pool: &SqlitePool,
        campaign_id: &str,
        system_prompt: &str,
        messages: Vec<ChatMessage>,
        game_state: &GameState,
    ) -> Result<AgentResult> {
        let available_tools = tools::tools_for_state(game_state);
        let anthropic_tools = convert_tools_to_anthropic(&available_tools);
        let mut current_messages = messages.clone();
        let mut tool_calls_made: Vec<ToolCallRecord> = vec![];
        let mut roll_request: Option<RollRequest> = None;

        for iteration in 0..MAX_TOOL_ITERATIONS {
            tracing::debug!("Agent loop iteration {}", iteration);

            let response = self.chat_completion(
                system_prompt,
                &current_messages,
                &anthropic_tools,
            ).await?;

            let stop_reason = response.stop_reason.as_deref().unwrap_or("end_turn");

            let mut narrative_text = String::new();
            let mut tool_uses: Vec<AnthropicToolUse> = vec![];

            for block in &response.content {
                match block.block_type.as_str() {
                    "text" => {
                        if let Some(text) = &block.text {
                            narrative_text = text.clone();
                        }
                    }
                    "tool_use" => {
                        if let (Some(id), Some(name), Some(input)) =
                            (&block.id, &block.name, &block.input)
                        {
                            tool_uses.push(AnthropicToolUse {
                                id: id.clone(),
                                name: name.clone(),
                                input: input.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }

            if stop_reason == "end_turn" || tool_uses.is_empty() {
                return Ok(AgentResult {
                    narrative: narrative_text,
                    tool_calls_made,
                    roll_request,
                });
            }

            current_messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: Some(narrative_text.clone()),
                anthropic_content: Some(response.content.clone()),
                tool_results: None,
            });

            for tu in &tool_uses {
                if tu.name == "request_roll" {
                    roll_request = Some(RollRequest {
                        tool_call_id: tu.id.clone(),
                        die: tu.input["die"].as_str().unwrap_or("d20").to_string(),
                        skill: tu.input["skill"].as_str().unwrap_or("").to_string(),
                        dc: tu.input["dc"].as_i64().unwrap_or(10),
                        reason: tu.input["reason"].as_str().unwrap_or("").to_string(),
                    });

                    return Ok(AgentResult {
                        narrative: narrative_text,
                        tool_calls_made,
                        roll_request,
                    });
                }
            }

            let mut tool_result_blocks: Vec<Value> = vec![];

            for tu in &tool_uses {
                tracing::info!("Executing tool: {} with args: {}", tu.name, tu.input);

                let result = executor::execute_tool(
                    pool,
                    campaign_id,
                    &tu.name,
                    &tu.input,
                ).await;

                let result_str = match result {
                    Ok(val) => serde_json::to_string_pretty(&val).unwrap_or_default(),
                    Err(e) => {
                        tracing::error!("Tool '{}' error: {}", tu.name, e);
                        format!("{{\"error\": \"{}\"}}", e)
                    }
                };

                tool_calls_made.push(ToolCallRecord {
                    tool_name: tu.name.clone(),
                    args: tu.input.clone(),
                    result: result_str.clone(),
                });

                tool_result_blocks.push(json!({
                    "type": "tool_result",
                    "tool_use_id": tu.id,
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
        let content = response.content.iter()
            .find(|b| b.block_type == "text")
            .and_then(|b| b.text.clone())
            .unwrap_or_else(|| "The attack lands.".to_string());
        Ok(content)
    }

    async fn chat_completion(
        &self,
        system: &str,
        messages: &[ChatMessage],
        tools: &[Value],
    ) -> Result<AnthropicResponse> {
        let anthropic_messages = build_anthropic_messages(messages);

        let mut payload = json!({
            "model": self.model,
            "max_tokens": 1024,
            "system": system,
            "messages": anthropic_messages,
        });

        if !tools.is_empty() {
            payload["tools"] = json!(tools);
        }

        let response = self.client
            .post(ANTHROPIC_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error {}: {}", status, body));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;
        Ok(anthropic_response)
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