use tokio_postgres::{Statement, Client};
use crate::DatabaseSettings;
use anyhow::{anyhow, Error};

pub struct StatementBuilder {}

impl StatementBuilder {

  // ================== Articles ==================

  pub async fn articles_statement(
    client: &Client,
    config: &DatabaseSettings,
  ) -> Result<Statement, Error> {
    let stmt = include_str!("../prepared_statements/articles/articles.sql");
    let stmt = client.prepare(stmt).await;

    match stmt {
      Ok(stmt) => Ok(stmt),
      Err(error) => {
        let error = anyhow!(
            "Failed to prepare articles statement: {} host: {:?}, user: {:?}, config{:?}",
            error,
            config.host,
            config.username,
            config
        );
        Err(error)
      }
    }
  }

  pub async fn article_statement(
    client: &Client,
    config: &DatabaseSettings,
  ) -> Result<Statement, Error> {
    let stmt = include_str!("../prepared_statements/articles/article.sql");
    let stmt = client.prepare(stmt).await;

    match stmt {
      Ok(stmt) => Ok(stmt),
      Err(error) => {
        let error = anyhow!(
            "Failed to prepare article statement: {} host: {:?}, user: {:?}, config{:?}",
            error,
            config.host,
            config.username,
            config
        );
        Err(error)
      }
    }
  }

  pub async fn article_upsert_statement(
    client: &Client,
    config: &DatabaseSettings,
  ) -> Result<Statement, Error> {
    let stmt = include_str!("../prepared_statements/articles/article_upsert.sql");
    let stmt = client.prepare(stmt).await;

    match stmt {
      Ok(stmt) => Ok(stmt),
      Err(error) => {
        let error = anyhow!(
            "Failed to prepare article upsert statement: {} host: {:?}, user: {:?}, config{:?}",
            error,
            config.host,
            config.username,
            config
        );
        Err(error)
      }
    }
  }

  pub async fn article_delete_statement(
    client: &Client,
    config: &DatabaseSettings,
  ) -> Result<Statement, Error> {
    let stmt = include_str!("../prepared_statements/articles/article_delete.sql");
    let stmt = client.prepare(stmt).await;

    match stmt {
      Ok(stmt) => Ok(stmt),
      Err(error) => {
        let error = anyhow!(
            "Failed to prepare article delete statement: {} host: {:?}, user: {:?}, config{:?}",
            error,
            config.host,
            config.username,
            config
        );
        Err(error)
      }
    }
  }

  // ================== Courses ==================

  pub async fn courses_statement(
    client: &Client,
    config: &DatabaseSettings,
  ) -> Result<Statement, Error> {
    let stmt = include_str!("../prepared_statements/courses/courses.sql");
    let stmt = client.prepare(stmt).await;

    match stmt {
      Ok(stmt) => Ok(stmt),
      Err(error) => {
        let error = anyhow!(
            "Failed to prepare courses statement: {} host: {:?}, user: {:?}, config{:?}",
            error,
            config.host,
            config.username,
            config
        );
        Err(error)
      }
    }
  }

  pub async fn course_statement(
    client: &Client,
    config: &DatabaseSettings,
  ) -> Result<Statement, Error> {
    let stmt = include_str!("../prepared_statements/courses/course.sql");
    let stmt = client.prepare(stmt).await;

    match stmt {
      Ok(stmt) => Ok(stmt),
      Err(error) => {
        let error = anyhow!(
            "Failed to prepare course statement: {} host: {:?}, user: {:?}, config{:?}",
            error,
            config.host,
            config.username,
            config
        );
        Err(error)
      }
    }
  }

  pub async fn course_upsert_statement(
    client: &Client,
    config: &DatabaseSettings,
  ) -> Result<Statement, Error> {
    let stmt = include_str!("../prepared_statements/courses/course_upsert.sql");
    let stmt = client.prepare(stmt).await;

    match stmt {
      Ok(stmt) => Ok(stmt),
      Err(error) => {
        let error = anyhow!(
            "Failed to prepare course upsert statement: {} host: {:?}, user: {:?}, config{:?}",
            error,
            config.host,
            config.username,
            config
        );
        Err(error)
      }
    }
  }

  pub async fn course_delete_statement(
    client: &Client,
    config: &DatabaseSettings,
  ) -> Result<Statement, Error> {
    let stmt = include_str!("../prepared_statements/courses/course_delete.sql");
    let stmt = client.prepare(stmt).await;

    match stmt {
      Ok(stmt) => Ok(stmt),
      Err(error) => {
        let error = anyhow!(
            "Failed to prepare course delete statement: {} host: {:?}, user: {:?}, config{:?}",
            error,
            config.host,
            config.username,
            config
        );
        Err(error)
      }
    }
  }

}