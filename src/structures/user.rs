use serde::Deserialize;
use sqlx::{Executor, MySql, Pool, Row};

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub privileges: i32,
    pub country: String,
}
impl User {
    pub async fn fetch_all(pool: &Pool<MySql>) -> Vec<User> {
        // Using pool to fetch all users
        let start = std::time::Instant::now();
        let rows = pool
            .fetch_all("SELECT id, name, priv, country FROM users")
            .await
            .unwrap()
            .into_iter();
        println!("Query took {:?}", std::time::Instant::now() - start);

        let users = rows
            .map(|row| User {
                id: row.get("id"),
                name: row.get("name"),
                privileges: row.get("priv"),
                country: row.get("country"),
            })
            .collect();

        return users;
    }
}
