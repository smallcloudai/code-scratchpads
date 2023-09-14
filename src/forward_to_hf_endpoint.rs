use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
// use eventsource_client::EventSource;
use reqwest_streams::JsonStreamResponse;
// use futures_util::stream::BoxStream;
use crate::call_validation::SamplingParameters;
use serde_json::json;
use futures::stream::Stream;


pub async fn forward_to_hf_style_endpoint(
    bearer: Option<String>,
    model_name: &str,
    prompt: &str,
    client: &reqwest::Client,
    endpoint_template: &String,
    sampling_parameters: &SamplingParameters,
    stream: bool,
) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<String, reqwest_streams::error::StreamBodyError>> + Send>>, String> {
// ) -> Result<impl Stream<Item = String>, String> {
    let url = endpoint_template.replace("$MODEL", model_name);
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
    if let Some(t) = bearer {
        headers.insert(AUTHORIZATION, HeaderValue::from_str(t.as_str()).unwrap());
    }
    let params_string = serde_json::to_string(sampling_parameters).unwrap();
    let mut params_json = serde_json::from_str::<serde_json::Value>(&params_string).unwrap();
    params_json["return_full_text"] = serde_json::Value::Bool(false);

    let data = json!({
        "inputs": prompt,
        "parameters": params_json,
        "stream": stream,
    });

    // let _stream: std::pin::Pin<Box<dyn Stream<Item = Result<_, reqwest_streams::error::StreamBodyError>> + Send>> = reqwest::get("http://localhost:8080/json-array")
    // let x = reqwest::get("http://localhost:8080/json-array")
    //     .await
    //     .map_err(|e| format!("when making request {}: {}", url, e))?
    //     .json_nl_stream(1024);
    let x = client.post(&url)
        .headers(headers)
        .body(data.to_string())
        .send()
        .await
        .map_err(|e| format!("when making request {}: {}", url, e))?
        .json_nl_stream(32768);
        // .bytes_stream();

    // let event_source = EventSource::new(client.post(&url))
    //     .header(headers)
    //     .data(data.to_string());

    // let stream = event_source
    //     .stream()
    //     .map_err(|e| format!("streaming events from {}: {}", url, e));

    // let req = client.post(&url)
    //    .headers(headers)
    //    .body(data.to_string())
    //    .send()
    //    .await;
    // let resp = req.map_err(|e| format!("when making request {}: {}", url, e))?;
    // let status_code = resp.status().as_u16();
    // let response_txt = resp.text().await.map_err(|e|
    //     format!("reading from socket {}: {}", url, e)
    // )?;
    // if status_code != 200 {
    //     return Err(format!("{} status={} text {}", url, status_code, response_txt));
    // }

    Ok(x)
}


// with streaming:
// use futures::stream::Stream;
// -> impl Stream<Item = String>
