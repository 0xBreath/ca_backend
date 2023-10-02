use postgres_client::configurations::DatabaseSettings;
use sqlx::{Connection, Executor, PgConnection, PgPool};

// pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
//     let mut connection = PgConnection::connect_with(&config.without_db())
//         .await
//         .expect("Failed to connect to Postgres");
//     connection
//         .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name.unwrap()))
//         .await
//         .expect("Failed to create database.");
//
//     let connection_pool = PgPool::connect_with(config.with_db())
//         .await
//         .expect("Failed to connect to Postgres.");
//     sqlx::migrate!("./migrations")
//         .run(&connection_pool)
//         .await
//         .expect("Failed to migrate the database");
//
//     connection_pool
// }
//
// #[tokio::test]
// async fn try_configure_db() -> Result<(), anyhow::Error> {
//     let mut config = postgres_client::configurations::get_configuration().expect("Failed to read configuration.");
//     config.database_name = Some("test".to_string());
//     let db = configure_database(&config).await;
//     Ok(())
// }
