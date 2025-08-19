use serde::{Deserialize, Serialize};
use sqlx::{
    Sqlite, SqlitePool, migrate::MigrateDatabase, prelude::FromRow, sqlite::SqliteConnectOptions,
};
use uuid::Uuid;

use crate::model::{conversation::Conversation, message::Message};

const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

#[derive(FromRow, Deserialize, Serialize, Debug, Clone)]
pub struct Emb {
    id: String,
    created_at: String,
    topic: String,
    embedding: String, // JSON string representation of the embedding
}

impl Database {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
            println!("Creating database {}", DB_URL);
            match Sqlite::create_database(DB_URL).await {
                Ok(_) => println!("Create db success"),
                Err(error) => panic!("error: {}", error),
            }
        } else {
            println!("Database already exists");
        }

        let pool = SqlitePool::connect(DB_URL).await?;

        // probably want all this in a migrations file(s)
        // so based on this
        // we store all the messages regardless of who sent them
        // then retrieve them based on the conversation_id
        //
        // get conversations with user_id
        //
        // get messages with conversation_id
        //
        // when create conversation, create
        //
        // when create message, insert into messages with conversation_id
        // this last one also applies to the robot
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT current_timestamp,
                sender_type TEXT,
                conversation_id TEXT NOT NULL,
                message TEXT
            );
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT current_timestamp,
                user_id TEXT NOT NULL,
                title TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS embeddings (
                id TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT current_timestamp,
                topic TEXT NOT NULL,
                embedding TEXT NOT NULL
            );
            ",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
    pub async fn add_message_to_conversation(
        self,
        sender_type: &str,
        conversation_id: &str,
        message: &str,
    ) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4().to_string(); // TODO: can i not use to_string() here?
        sqlx::query(
            "INSERT INTO messages (id, sender_type, conversation_id, message) VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind(sender_type)
        .bind(conversation_id)
        .bind(message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    pub async fn create_conversation(
        &self,
        user_id: &str,
        title: &str,
    ) -> Result<String, sqlx::Error> {
        let id = Uuid::new_v4().to_string(); // TODO: can i not use to_string() here?
        sqlx::query("INSERT INTO conversations (id, user_id, title) VALUES (?, ?, ?)")
            .bind(&id)
            .bind(user_id) // replace with actual user_id
            .bind(title) // replace with actual title
            .execute(&self.pool)
            .await?;

        Ok(id)
    }
    pub async fn get_conversation_list_by_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<Conversation>, sqlx::Error> {
        let result = sqlx::query_as::<_, Conversation>(
            "SELECT * FROM conversations WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(result)
    }
    pub async fn get_conversation_by_id(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<Message>, sqlx::Error> {
        // TODO: this doesn't need to return an array
        let result = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC",
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_conversation(&self, conversation_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(conversation_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    pub async fn create_embedding(
        &self,
        topic: String,
        embedding: Vec<(String, f64)>,
    ) -> Result<(), sqlx::Error> {
        let uuid = Uuid::new_v4().to_string();

        let embedding =
            serde_json::to_string(&embedding).map_err(|e| sqlx::Error::Decode(Box::new(e)))?; // Convert embedding to JSON string

        sqlx::query("INSERT INTO embeddings (id, topic, embedding) VALUES (?, ?, ?)")
            .bind(uuid)
            .bind(topic)
            .bind(embedding)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_embedding_by_topic(
        &self,
        topic: &str,
    ) -> Result<Option<Vec<(String, f64)>>, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, Emb>("SELECT embedding FROM embeddings WHERE topic = ?")
            .bind(topic)
            .fetch_one(&self.pool)
            .await?;
        let thing: Vec<(String, f64)> = serde_json::from_str(&result.embedding)?;
        Ok(Some(thing))
    }

    pub async fn get_embedding_by_id(
        &self,
        id: &str,
    ) -> Result<Option<Vec<(String, f64)>>, Box<dyn std::error::Error>> {
        let result = sqlx::query_as::<_, Emb>("SELECT embedding FROM embeddings WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        let thing: Vec<(String, f64)> = serde_json::from_str(&result.embedding)?;
        Ok(Some(thing))
    }
}
