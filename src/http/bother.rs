use axum::{http::StatusCode, response::IntoResponse};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::consts::{ASSISTANT, LANGUAGE_MODEL, ROBOT_NAME, SYSTEM, USER};
use crate::db::database::Database;
use crate::model::bother::Bother;
use crate::vector::embedding::Embedding;

#[derive(Serialize, Deserialize, Debug)]
struct RobotMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RobotChoice {
    finish_reason: String,
    index: usize,
    message: RobotMessage,
}

#[derive(Serialize, Deserialize, Debug)]
struct RobotResponses {
    choices: Vec<RobotChoice>,
}

pub async fn bother_blockito(
    axum::Extension(db): axum::Extension<Arc<Database>>,
    axum::Extension(embedding): axum::Extension<Arc<Embedding>>,
    axum::Json(bother): axum::Json<Bother>,
) -> impl IntoResponse {
    println!("Bother received {:?}", bother.message);
    // basically, hit the embedding model
    // use that to send to the language model
    // everything else should work as expected
    // (it won't but we can pretent for now)

    let retrieved_knowledge = embedding
        .retrieve(bother.message.as_str(), None)
        .await
        .unwrap(); // TODO remove unwrap, probably have this use result

    for (chunk, similarity) in &retrieved_knowledge {
        println!("Found chunk: {chunk} with similarity: {similarity}");
    }

    let context: String = retrieved_knowledge
        .iter()
        .map(|(chunk, _)| format!(" - {}", chunk))
        .collect::<Vec<_>>()
        .join("\n");
    let instruction_prompt = format!(
        "You are a helpful chatbot named {ROBOT_NAME}.\nUse only the following pieces of context to answer the question. Don't make up any new information:\n{context} /no_think",
    );
    println!("Instruction prompt: {}", instruction_prompt);

    // if no conversation_id, create a new conversation
    // store initial message
    // store llama response
    // return as expected

    let conversation_id = if bother.conversation_id.is_none() {
        // TODO: remove unwrap
        db.insert_conversation(&bother.user_id, "temp title, we'll figure this out later")
            .await
            .unwrap()
    } else {
        bother.conversation_id.unwrap()
    };
    println!("Conversation ID: {}", conversation_id);
    let conversation = db.get_conversation_by_id(&conversation_id).await.unwrap(); // TODO: remove unwrap
    println!("Conversation: {:?}", conversation);

    // I actually think this is wrong...
    // this will alter the sysm prompt each time
    // which i don't think is right
    let mut messages = vec![json!({
        "role": SYSTEM,
        "content": instruction_prompt,
    })];

    for message in conversation {
        messages.push(json!({
            "role": message.sender_type,
            "content": message.message
        }));
    }

    // TODO: sort out db work
    // db.add_message_to_conversation(USER);

    // get the conversation messages from the database
    // TODO: chat history should be stored/added
    // TODO: push in all historical

    let request = json!({
        "model": LANGUAGE_MODEL,
        // "prompt": bother.message,
        "messages": messages,
    });

    let client = reqwest::Client::new();
    let body = client
        .post("http://127.0.0.1:8765/v1/chat/completions")
        .json(&request)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap(); // TODO: no unwrap
    let parsed_body: RobotResponses = serde_json::from_str(&body).unwrap(); // TODO: no unwrap
    // println!("Parsed response: {:?}", parsed_body);
    // TODO: consider combining if more than one choice
    let message = parsed_body.choices[0].message.content.clone();
    // TODO: add the response message to the conversation list (in the db)
    // TODO: parse the response and return something useful
    // TODO: look into streaming
    // TODO: something something full chat response
    (
        StatusCode::OK,
        axum::Json(json!({
            "message": message
        })),
    )
}
