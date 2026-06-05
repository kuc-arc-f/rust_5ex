use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// リクエストボディの構造体
#[derive(Serialize)]
struct EmbedContentRequest {
    model: String,
    content: Content,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

/// レスポンスボディの構造体
#[derive(Deserialize, Debug)]
struct EmbedContentResponse {
    embedding: Embedding,
}

#[derive(Deserialize, Debug)]
struct Embedding {
    values: Vec<f32>,
}

/// Gemini Embedding API を呼び出してベクトルを取得する関数
pub async fn get_embedding(api_key: &str, text: &str) -> anyhow::Result<Vec<f32>> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-001:embedContent?key={}",
        api_key
    );

    let request_body = EmbedContentRequest {
        model: "models/gemini-embedding-001".to_string(),
        content: Content {
            parts: vec![Part {
                text: text.to_string(),
            }],
        },
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .context("APIへのリクエスト送信に失敗しました")?;


    // HTTPステータスコードの確認
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("APIエラー ({}): {}", status, error_text);
    }

    // レスポンスをデシリアライズ
    let embed_response: EmbedContentResponse = response
        .json()
        .await
        .context("レスポンスのJSONパースに失敗しました")?;

    //return embed_response.embedding.values;
    Ok(embed_response.embedding.values)
}