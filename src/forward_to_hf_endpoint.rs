use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde_json::json;


pub async fn simple_forward_to_hf_endpoint_no_streaming(
    model_name: &str,
    prompt: &str,
    client: &reqwest::Client,
    bearer: Option<String>,
    // sampling_parameters: &Dict<String, Any>,
    // stream: bool,
) -> Result<serde_json::Value, serde_json::Error> {
    let url = format!("https://api-inference.huggingface.co/models/{}", model_name);
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
    if let Some(t) = bearer {
        headers.insert(AUTHORIZATION, HeaderValue::from_str(t.as_str()).unwrap());
    }
    let data = json!({
        "inputs": prompt,
        "parameters": {
            "return_full_text": false,
        },
        // "stream": stream,
    });
    let response = client.post(&url)
       .headers(headers)
       .body(data.to_string())
       .send()
       .await;
    let response_txt = response.unwrap().text().await.unwrap();
    Ok(serde_json::from_str(&response_txt).unwrap())
}


// with streaming:
// use futures::stream::Stream;
// -> impl Stream<Item = String>

