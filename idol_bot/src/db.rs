use anyhow::Result;
use async_std::prelude::*;
use log::*;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

pub struct Webhook {
    pub url: String,
}

impl Database {
    pub async fn connect(uri: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new().connect(uri).await?;

        let migrator = sqlx::migrate!("./migrations");
        migrator.run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn webhooks(&self) -> impl Stream<Item = Result<Webhook>> + '_ {
        sqlx::query_as!(Webhook, "SELECT url FROM webhooks")
            .fetch(&self.pool)
            .map(|x| Ok(x?))
    }

    pub async fn count(&self) -> Result<i32> {
        Ok(sqlx::query!("SELECT COUNT(*) as count FROM webhooks")
            .fetch_one(&self.pool)
            .await?
            .count)
    }

    pub async fn add_urls(&self, urls: impl Iterator<Item = &str>) -> Result<()> {
        let mut transaction = self.pool.begin().await?;
        for url in urls {
            debug!("adding URL: {:?}", url);
            sqlx::query!("INSERT OR IGNORE INTO webhooks (url) VALUES (?)", url)
                .execute(&mut transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn add_url(&self, url: &str) -> Result<()> {
        self.add_urls(std::iter::once(url)).await?;
        Ok(())
    }

    pub async fn remove_url(&self, url: &str) -> Result<()> {
        sqlx::query!("DELETE FROM webhooks WHERE url = ?", url)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
