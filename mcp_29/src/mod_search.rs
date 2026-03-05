use dotenvy::dotenv;
use qdrant_client::qdrant::{
    Condition, CreateCollectionBuilder, Distance, Filter, PointStruct, ScalarQuantizationBuilder,
    SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::{Payload, Qdrant, QdrantError};
use reqwest::Client;
use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use std::fmt;
use std::env;
use std::fs;
use std::path::Path;
use std::io::{self, Read};


#[derive(Deserialize, Debug)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub values: Vec<f32>,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}


// エンベディング結果を格納する構造体
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub embedding: Vec<f32>,
}


/**
*
* @param
*
* @return
*/
pub async fn EmbedUserQuery(query :String) -> Vec<f32> {
    let items : Vec<f32> = Vec::new();
    let client = reqwest::Client::new();

    let request = EmbeddingRequest {
        model: "qwen3-embedding:0.6b".to_string(),
        prompt: query.to_string(),
    };

    let res = client
        .post("http://localhost:11434/api/embeddings")
        .json(&request)
        .send()
        .await.unwrap();
    println!("Status: {:?}", res.status());

    if res.status().is_success() {
        let response_body: EmbeddingResponse = res.json().await.unwrap();
        println!("Embedding length: {}", response_body.embedding.len());
        // Print first few elements to verify without flooding console
        if response_body.embedding.len() > 0 {
             println!("dimensions.len: {}", response_body.embedding.len());
             return response_body.embedding;
         } else {
             println!("Embedding: {:?}", response_body.embedding);
        }
       
    } else {
        println!("Request failed: {:?}", res.status());
        let text = res.text().await.unwrap();
        println!("Response text: {}", text);
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
    dotenv().ok();
    let clientQdrant = Qdrant::from_url("http://localhost:6334").build().unwrap();

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EmbedItem {
        name: String,
        content: String,
        embeddings: Vec<u8>
    }
    let input_f32 = EmbedUserQuery(query.clone()).await;
    println!("input_f32.len={}", input_f32.len());
    let search_result = clientQdrant
        .search_points(
            SearchPointsBuilder::new(super::COLLECT_NAME, input_f32, 1)
                .with_payload(true)
                .params(SearchParamsBuilder::default().exact(true)),
        )
        .await.unwrap();
    //dbg!(&search_result);

    let resplen = search_result.result.len();
    println!("#list-start={}", resplen);
    println!("\nコサイン距離による類似検索結果:");
    let mut matches : String = "".to_string();
    let mut out_str : String = "".to_string();
    for row_resp in &search_result.result {
        let content = &row_resp.payload["content"];
        let content_str = format!("{}\n\n", content);
        matches.push_str(&content_str.clone().to_string());
    }
    //println!("matches={}\n", &matches);
    if matches.len() > 0 {
        out_str = format!("context: {}\n", matches);
        let out_add2 = format!("user query: {}\n" , query);
        out_str.push_str(&out_add2);
    }else {
        out_str = format!("user query: {}\n", query);
    }

    return out_str.to_string();
}