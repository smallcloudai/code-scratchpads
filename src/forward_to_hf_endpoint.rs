use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest_eventsource::EventSource;
use serde_json::json;
use crate::call_validation::SamplingParameters;

// Idea: use USER_AGENT
// let user_agent = format!("{NAME}/{VERSION}; rust/unknown; ide/{ide:?}");


pub async fn forward_to_hf_style_endpoint(
    save_url: &String,
    bearer: String,
    prompt: &str,
    client: &reqwest::Client,
    sampling_parameters: &SamplingParameters,
) -> Result<serde_json::Value, String> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
    if !bearer.is_empty() {
        headers.insert(AUTHORIZATION, HeaderValue::from_str(format!("Bearer {}", bearer).as_str()).unwrap());
    }
    let params_string = serde_json::to_string(sampling_parameters).unwrap();
    let mut params_json = serde_json::from_str::<serde_json::Value>(&params_string).unwrap();
    params_json["return_full_text"] = serde_json::Value::Bool(false);

    let data = json!({
        "inputs": prompt,
        "parameters": params_json,
    });
    let req = client.post(save_url)
        .headers(headers)
        .body(data.to_string())
        .send()
        .await;
    let resp = req.map_err(|e| format!("{}", e))?;
    let status_code = resp.status().as_u16();
    let response_txt = resp.text().await.map_err(|e|
        format!("reading from socket {}: {}", save_url, e)
    )?;
    if status_code != 200 {
        return Err(format!("{} status={} text {}", save_url, status_code, response_txt));
    }
    Ok(serde_json::from_str(&response_txt).unwrap())
}


pub async fn forward_to_hf_style_endpoint_streaming(
    save_url: &String,
    bearer: String,
    prompt: &str,
    client: &reqwest::Client,
    sampling_parameters: &SamplingParameters,
) -> Result<EventSource, String> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
    if !bearer.is_empty() {
        headers.insert(AUTHORIZATION, HeaderValue::from_str(format!("Bearer {}", bearer).as_str()).unwrap());
    }
    let params_string = serde_json::to_string(sampling_parameters).unwrap();
    let mut params_json = serde_json::from_str::<serde_json::Value>(&params_string).unwrap();
    params_json["return_full_text"] = serde_json::Value::Bool(false);

    let data = json!({
        "inputs": prompt,
        "parameters": params_json,
        "stream": true,
    });

    let builder = client.post(save_url)
       .headers(headers)
       .body(data.to_string());
    let event_source: EventSource = EventSource::new(builder).map_err(|e|
        format!("can't stream from {}: {}", save_url, e)
    )?;
    Ok(event_source)
}
