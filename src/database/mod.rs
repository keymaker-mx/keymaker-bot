use crate::config::Config;
use crate::errors::Error;
use sqlx::postgres::PgPool;
use tracing::*;

pub mod models;

#[cfg(test)]
#[instrument(skip(config))]
pub async fn get_database_pool<'a>(config: Config<'a>) -> Result<PgPool, Error>
where
    Config<'a>: mrsbfh::config::Loader + Clone,
{
    println!("test");
    use std::env;
    Ok(PgPool::connect(&env::var("DATABASE_URL")?).await?)
}

// This is a singleton
#[cfg(not(test))]
#[instrument(skip(config))]
pub async fn get_database_pool<'a>(config: Config<'a>) -> Result<PgPool, Error>
where
    Config<'a>: mrsbfh::config::Loader + Clone,
{
    use once_cell::sync::OnceCell;
    static INSTANCE: OnceCell<PgPool> = OnceCell::new();
    if let Some(pool) = INSTANCE.get() {
        Ok(pool.clone())
    } else {
        let pool = PgPool::connect(&config.database_url).await?;
        if INSTANCE.set(pool).is_err() {
            return Err(Error::DatabaseSingletonError);
        }
        Ok(INSTANCE.get().unwrap().clone())
    }
}
