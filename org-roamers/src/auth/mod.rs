pub mod password;
pub mod session_store;

use std::collections::HashMap;

use crate::config::Config;

/// User definition from config
#[derive(Clone)]
pub struct User {
    pub username: String,
    pub password: String,
}

/// Stores hashed passwords for user authentication
/// Built from config at startup
pub struct UserStore {
    /// Map of username -> argon2id hash
    users: HashMap<String, String>,
}

impl UserStore {
    pub fn from_users(users: Vec<User>) -> Result<Self, argon2::password_hash::Error> {
        use tracing::info;

        let mut user_map = HashMap::new();

        for user in users {
            info!("Hashing password for user: {}", user.username);
            let hash = password::hash_password(&user.password)?;
            user_map.insert(user.username, hash);
        }

        info!("Loaded {} user(s) for authentication", user_map.len());

        Ok(Self { users: user_map })
    }

    pub fn verify(&self, username: &str, password: &str) -> bool {
        match self.users.get(username) {
            Some(hash) => password::verify_password(password, hash).unwrap_or(false),
            None => false,
        }
    }

    pub fn user_exists(&self, username: &str) -> bool {
        self.users.contains_key(username)
    }

    pub fn user_count(&self) -> usize {
        self.users.len()
    }
}

pub fn build_user_store(conf: &Config) -> anyhow::Result<Option<UserStore>> {
    if let Some(auth_config) = &conf.authentication {
        if auth_config.enabled {
            tracing::info!("Authentication enabled, loading users...");

            // Convert config Users to auth Users
            let users: Vec<crate::auth::User> = auth_config
                .users
                .iter()
                .map(|u| crate::auth::User {
                    username: u.username.clone(),
                    password: u.password.clone(),
                })
                .collect();

            let store = UserStore::from_users(users)
                .map_err(|e| anyhow::anyhow!("Failed to hash passwords: {}", e))?;
            Ok(Some(store))
        } else {
            tracing::info!("Authentication disabled in config");
            Ok(None)
        }
    } else {
        tracing::info!("No authentication configuration found, running without auth");
        Ok(None)
    }
}
