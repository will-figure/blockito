use reqwest;
use serde::Deserialize;
use serde_json::json;
use std::fs;

use crate::consts::{EMBEDDING_MODEL, LLAMA_URL};

#[derive(Deserialize, Debug)]
struct EmbeddingsResponseData {
    pub embedding: Vec<f64>,
}

#[derive(Deserialize, Debug)]
struct EmbeddingsResponse {
    data: Vec<EmbeddingsResponseData>,
}

#[derive(Debug, Clone)]
pub struct Embedding {
    vector_db: Vec<(String, Vec<f64>)>, // not sure if string is correct yet
}

// TODO: real erorr handling
impl Embedding {
    pub async fn new() -> anyhow::Result<Self> {
        let mut vector_db = vec![];
        let file = fs::read_to_string("cat-facts.txt")
            .expect("something bad happened while loading the data");

        for line in file.lines() {
            let vector = Self::embedding(line).await?;
            vector_db.push((line.to_string(), vector));
        }
        println!(
            "Vector database initialized with {} entries",
            vector_db.len()
        );
        Ok(Embedding { vector_db })
    }
    async fn embedding(line: &str) -> anyhow::Result<Vec<f64>> {
        // TODO: consider reusing the client
        let client = reqwest::Client::new();
        let request = json!({
            "input": line,
            "model": EMBEDDING_MODEL,
        });
        let embedding_result = client
            .post(format!("{LLAMA_URL}/v1/embeddings")) // replace with actual embedding model URL
            .json(&request)
            .send()
            .await?
            .json::<EmbeddingsResponse>()
            .await?;

        let vector = embedding_result.data[0].embedding.clone();
        Ok(vector)
    }
    fn cosign_similarity(a: &[f64], b: &[f64]) -> f64 {
        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        dot_product / (norm_a * norm_b)
    }
    pub async fn retrieve(
        &self,
        query: &str,
        top_n: Option<usize>,
    ) -> anyhow::Result<Vec<(String, f64)>> {
        let query_embedding = Self::embedding(query).await?;
        let mut similarities = Vec::new();

        for (chunk, emedding) in self.vector_db.iter() {
            let similarity = Self::cosign_similarity(&query_embedding, emedding);
            similarities.push((chunk.to_string(), similarity));
        }
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top_n = top_n.unwrap_or(5);

        let top_similarities = &similarities[..top_n];

        let result = top_similarities
            .iter()
            .map(|(chunk, similarity)| (chunk.clone(), *similarity))
            .collect::<Vec<_>>();

        Ok(result)
    }
}
