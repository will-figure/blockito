use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase, prelude::FromRow};
use uuid::Uuid;

use crate::model::{conversation::Conversation, message::Message};

const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

#[derive(FromRow, Deserialize, Serialize, Debug, Clone)]
pub struct StoredEmbedding {
    id: String,
    created_at: String,
    topic: String,
    embedding: String, // JSON string representation of the embedding
}

pub struct ParsedEmbedding {
    id: String,
    created_at: String,
    topic: String,
    embedding: Vec<(String, f64)>,
}

// we don't need to send the actual embedding back
pub struct EmbeddingMetadata {
    pub id: String,
    pub created_at: String,
    pub topic: String,
}

// TODO: error consistency (move to anyhow)
impl Database {
    pub async fn new() -> anyhow::Result<Self> {
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
        // TODO: actual migrations
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
        &self,
        sender_type: &str,
        conversation_id: &str,
        message: &str,
    ) -> anyhow::Result<()> {
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
    pub async fn insert_conversation(&self, user_id: &str, title: &str) -> anyhow::Result<String> {
        let id = Uuid::new_v4().to_string(); // TODO: can i not use to_string() here?
        println!("Inserting conversation with id: {id} and user_id: {user_id}");
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
    ) -> anyhow::Result<Vec<Conversation>> {
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
    ) -> anyhow::Result<Vec<Message>> {
        // TODO: this doesn't need to return an array
        let result = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC",
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_conversation(&self, conversation_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(conversation_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // eventually behind some permissions
    pub async fn insert_embedding(
        &self,
        topic: String,
        embedding: Vec<(String, f64)>,
    ) -> anyhow::Result<()> {
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

    // eventually behind some permissions
    pub async fn delete_embedding(&self, id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM embeddings WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_embedding_by_id(&self, id: &str) -> anyhow::Result<Option<ParsedEmbedding>> {
        let result =
            sqlx::query_as::<_, StoredEmbedding>("SELECT embedding FROM embeddings WHERE id = ?")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        let embedding: Vec<(String, f64)> = serde_json::from_str(&result.embedding)?;

        let parsed_embedding = ParsedEmbedding {
            id: result.id,
            created_at: result.created_at,
            topic: result.topic,
            embedding,
        };

        Ok(Some(parsed_embedding))
    }

    // going to need to figure out the best way to pair this to a conversation...
    pub async fn get_all_embeddings(&self) -> anyhow::Result<Vec<EmbeddingMetadata>> {
        let results = sqlx::query_as::<_, StoredEmbedding>("SELECT * FROM embeddings")
            .fetch_all(&self.pool)
            .await?;

        let mut parsed_embeddings = Vec::new();
        for result in results {
            parsed_embeddings.push(EmbeddingMetadata {
                id: result.id,
                created_at: result.created_at,
                topic: result.topic,
            });
        }

        Ok(parsed_embeddings)
    }
}
