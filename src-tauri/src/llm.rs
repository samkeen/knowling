use babel_bridge::client::AnthropicClient;
use babel_bridge::error::ApiError;
use babel_bridge::models::{Message, ResponseMessage};

const DEFAULT_MODEL: &str = "claude-3-haiku-20240307";
const DEFAULT_MAX_TOKENS: u32 = 1000;
const DEFAULT_TEMP: f32 = 1.0;

pub async fn llm_request(prompt: &str, api_key: &str, system_prompt: Option<&str>)
                         -> Result<ResponseMessage, ApiError> {
    let client = AnthropicClient::new(api_key.to_string());
    let messages = vec![Message {
        role: "user".to_string(),
        content: prompt.to_string(),
    }];

    let mut request = client.request()
        .messages(messages)
        .model(DEFAULT_MODEL)
        .max_tokens(DEFAULT_MAX_TOKENS)
        .temperature(DEFAULT_TEMP);
    // apply system prompt if given
    if let Some(prompt) = system_prompt {
        request = request.system_prompt(prompt);
    }
    let response = request
        .send()
        .await?;

    Ok(response)
}