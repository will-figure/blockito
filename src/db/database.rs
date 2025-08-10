use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};
use uuid::Uuid;

use crate::model::{conversation::Conversation, message::Message};

const DB_URL: &str = "sqlite://sqlite.db";

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: SqlitePool,
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
}
