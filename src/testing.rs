use sqlx::{Row, SqlitePool};

pub async fn fetch_profile(
    username: &str,
    pool: &SqlitePool,
) -> Result<(String, String), anyhow::Error> {
    let recs = sqlx::query(
        r#"
        SELECT username, fullname
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_all(pool)
    .await?;

    if recs.is_empty() {
        return Err(anyhow::anyhow!("No user found"));
    }

    Ok((username.to_string(), recs[0].get("fullname")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn username() -> &'static str {
        "zelda"
    }

    #[fixture]
    async fn db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE users (
                username TEXT PRIMARY KEY,
                fullname TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[fixture]
    async fn db_with_user(username: &str, #[future] db: SqlitePool) -> SqlitePool {
        let db = db.await;

        sqlx::query(
            r#"
            INSERT INTO users (username, fullname)
            VALUES ($1, $2)
            "#,
        )
        .bind(username.to_string())
        .bind(format!("Test User {username}"))
        .execute(&db)
        .await
        .unwrap();

        db
    }

    #[rstest]
    #[tokio::test]
    async fn test_fetch_profile(username: &str, #[future] db_with_user: SqlitePool) {
        let db = db_with_user.await;

        let res = fetch_profile(username, &db).await;
        assert!(res.is_ok());

        let res = fetch_profile("link", &db).await;
        assert!(res.is_err());
    }
}
