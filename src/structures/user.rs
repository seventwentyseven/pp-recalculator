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
    pub fn new(id: i32, name: String, privileges: i32, country: String) -> Self {
        Self {
            id,
            name,
            privileges,
            country,
        }
    }

    pub async fn fetch_all(pool: &Pool<MySql>) -> Vec<User> {
        // Using pool to fetch all users
        let rows = pool
            .fetch_all("SELECT id, name, priv, country FROM users")
            .await
            .unwrap();

        let mut users = Vec::new();

        for row in rows {
            let user = User {
                id: row.try_get("id").unwrap(),
                name: row.try_get("name").unwrap(),
                privileges: row.try_get("priv").unwrap(),
                country: row.try_get("country").unwrap(),
            };
            users.push(user);
        }

        return users;
    }
}
