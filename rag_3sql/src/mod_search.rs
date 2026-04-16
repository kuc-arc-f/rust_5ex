use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct EmbeddingRequest {
    input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<String>,
}

#[derive(Deserialize, Debug)]
struct EmbeddingData {
    object: String,
    embedding: Vec<f32>,
    index: u32,
}

#[derive(Deserialize, Debug)]
struct EmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct EmbeddingResponse {
    object: String,
    data: Vec<EmbeddingData>,
    model: String,
    usage: EmbeddingUsage,
}


/**
*
* @param
*
* @return
*/
pub async fn EmbedUserQuery(query :String) -> Vec<f32> {
    let items : Vec<f32> = Vec::new();

    // llama-server のエンドポイント（デフォルトポート: 8080）
    let server_url = "http://localhost:8080/v1/embeddings";

    // 埋め込みを取得したいテキスト
    let text = query;

    println!("=== llama-server 埋め込みクライアント ===");
    println!("送信テキスト: {}", text);
    println!("エンドポイント: {}", server_url);
    println!();

    let client = reqwest::Client::new();

    let request_body = EmbeddingRequest {
        input: text.to_string(),
        encoding_format: Some("float".to_string()),
    };

    println!("リクエスト送信中...");

    let response = client
        .post(server_url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await.unwrap();

    let status = response.status();
    println!("HTTPステータス: {}", status);

    if !status.is_success() {
        let error_body = response.text().await.unwrap();
        eprintln!("エラーレスポンス: {}", error_body);
        //return Err(format!("サーバーエラー: {}", status).into());
        return items;
    }

    let embedding_response: EmbeddingResponse = response.json().await.unwrap();

    println!("\n=== レスポンス ===");
    println!("モデル: {}", embedding_response.model);
    println!("オブジェクト: {}", embedding_response.object);
    println!("使用トークン数:");
    println!("  プロンプト: {}", embedding_response.usage.prompt_tokens);
    println!("  合計:       {}", embedding_response.usage.total_tokens);

    for data in &embedding_response.data {
        let vec = &data.embedding;
        println!("\n--- 埋め込みベクトル [index: {}] ---", data.index);
        println!("次元数: {}", vec.len());
        println!("先頭10要素: {:?}", &vec[..vec.len().min(10)]);
        return data.embedding.clone();
        // コサイン類似度のためのノルム計算
        //let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        //println!("L2ノルム: {:.6}", norm);
    }

    return items;
}


/**
*
* @param
*
* @return
*/
pub async fn CheckSimalirity(query: String) -> String {

    let input_f32 = EmbedUserQuery(query.clone()).await;
    println!("input_f32.len={}", input_f32.len());
    return "".to_string();
    let mut matches : String = "".to_string();
    let mut out_str : String = "".to_string();

    return out_str.to_string();
}
