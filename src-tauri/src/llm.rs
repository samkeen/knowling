use llm_api_adapter::client::AnthropicClient;
use llm_api_adapter::error::AnthropicError;
use llm_api_adapter::models::{Message, ResponseMessage};

pub async fn llm_request(prompt: &str, api_key: &str) -> Result<ResponseMessage, AnthropicError> {
    let client = AnthropicClient::new(api_key.to_string());

    let messages = vec![Message {
        role: "user".to_string(),
        content: prompt.to_string(),
    }];

    let response = client
        .send_message("claude-3-haiku-20240307", messages, 100, 1.0)
        .await?;

    Ok(response)
}