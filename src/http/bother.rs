use axum::{http::StatusCode, response::IntoResponse};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::consts::{ASSISTANT, LANGUAGE_MODEL, LLAMA_URL, ROBOT_NAME, SYSTEM, USER};
use crate::db::database::Database;
use crate::http::error::AppError;
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
) -> Result<impl IntoResponse, AppError> {
    println!("Bother received {:?}", bother.message);
    // basically, hit the embedding model
    // use that to send to the language model
    // everything else should work as expected
    // (it won't but we can pretent for now)

    let retrieved_knowledge = embedding.retrieve(bother.message.as_str(), None).await?;

    for (chunk, similarity) in &retrieved_knowledge {
        println!("Found chunk: {chunk} with similarity: {similarity}");
    }

    let context: String = retrieved_knowledge
        .iter()
        .map(|(chunk, _)| format!(" - {}", chunk))
        .collect::<Vec<_>>()
        .join("\n");

    let instruction_prompt = format!(
        "You are a helpful chatbot named {ROBOT_NAME}.\nUse only the following pieces of context to answer the question. Do not reponsed with nothing. Don't make up any new information:\n{context} /no_think",
    );

    // println!("Instruction prompt: {}", instruction_prompt);

    // if no conversation_id, create a new conversation
    // store initial message
    // store llama response
    // return as expected

    println!("Bother conversation_id: {:?}", &bother.conversation_id);
    let conversation_id = if bother.conversation_id.is_none() {
        println!("No conversation_id, creating new conversation");
        db.insert_conversation(&bother.user_id, "temp title, we'll figure this out later")
            .await?
    } else {
        // can unwrap here because we check above
        bother.conversation_id.unwrap()
    };
    println!("Conversation ID: {}", conversation_id);
    // TODO: should this return the message or the whole conversation up to this point?
    // let conversation = db.get_conversation_by_id(&conversation_id).await?;
    // println!("Conversation: {:?}", conversation);

    // I actually think this is wrong...
    // this will alter the sysm prompt each time
    // which i don't think is right
    let mut messages = vec![json!({
        "role": SYSTEM,
        "content": instruction_prompt,
    })];

    db.add_message_to_conversation(USER, &conversation_id, &bother.message)
        .await?;
    messages.push(json!({
        "role": USER,
        "content": bother.message
    }));

    // get the conversation messages from the database
    // TODO: chat history should be stored/added
    // TODO: push in all historical, might not be great... at least not with with small model
    let request = json!({
        "model": LANGUAGE_MODEL,
        "messages": messages,
    });

    let client = reqwest::Client::new();
    let body = client
        .post(format!("{LLAMA_URL}/v1/chat/completions"))
        .json(&request)
        .send()
        .await?
        .text()
        .await?;
    let parsed_body: RobotResponses = serde_json::from_str(&body)?;
    // TODO: consider combining if more than one choice
    let message = parsed_body.choices[0].message.content.clone();
    // might want to create and store the message id instead of letting the db do it
    // would be able to send to FE easier
    db.add_message_to_conversation(ASSISTANT, &conversation_id, &message)
        .await?;

    // TODO: look into streaming
    // TODO: something something full chat response
    Ok((
        StatusCode::OK,
        axum::Json(json!({
            "message": message,
            "conversationId": conversation_id
        })),
    ))
}
