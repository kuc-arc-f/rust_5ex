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

/*
#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .context("環境変数 GEMINI_API_KEY が設定されていません")?;

    let text = "What is the meaning of life?".to_string();

    println!("テキストをエンベッドします: {}", text);

    let embedding_values = get_embedding(&api_key, &text).await?;

    println!("取得成功! ベクトル次元数: {}", embedding_values.len());
    if embedding_values.len() >= 5 {
        println!("最初の5次元: {:?}", &embedding_values[..5]);
    }

    Ok(())
}
*/

/// Gemini Embedding API を呼び出してベクトルを取得する関数
//async fn get_embedding(api_key: &str, text: &str) -> Result<Vec<f32>> {
pub async fn GetEmbedding(api_key: &str, text: &str) -> Vec<f32> {
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
        .context("APIへのリクエスト送信に失敗しました").unwrap();

    // HTTPステータスコードの確認
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        //anyhow::bail!("APIエラー ({}): {}", status, error_text);
        println!("APIエラー ({}): {}", status, error_text);
    }

    // レスポンスをデシリアライズ
    let embed_response: EmbedContentResponse = response
        .json()
        .await
        .context("レスポンスのJSONパースに失敗しました").unwrap().clone();

    //Ok(embed_response.embedding.values)
    return embed_response.embedding.values;
}