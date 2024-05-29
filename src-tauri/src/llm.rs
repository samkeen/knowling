use llm_bridge::client::{ClientLlm, LlmClient};
use llm_bridge::error::ApiError;
use llm_bridge::models::ResponseMessage;

const DEFAULT_MAX_TOKENS: u32 = 1000;
const DEFAULT_TEMP: f32 = 1.0;

pub async fn llm_request(prompt: &str, api_key: &str, system_prompt: Option<&str>)
                         -> Result<ResponseMessage, ApiError> {
    let client_type = ClientLlm::Anthropic;
    let mut client = LlmClient::new(client_type, api_key.to_string());
    let mut request = client.request()
        .user_message(prompt)
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