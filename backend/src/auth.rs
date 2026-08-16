//! Authentication module — user management with MongoDB.
//! Handles signup, signin, and JWT token creation/verification.

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use mongodb::{bson::doc, Client, Collection};
use serde::{Deserialize, Serialize};

/// User document in MongoDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub created_at: i64,
    #[serde(default)]
    pub games_played: u32,
    #[serde(default)]
    pub games_won: u32,
    #[serde(default)]
    pub total_score: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reset_token_expires: Option<i64>,
}

/// JWT claims payload.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user email
    pub name: String,
    pub exp: usize,
    pub iat: usize,
}

/// Auth result sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub email: String,
    pub display_name: String,
}

pub struct AuthService {
    pub users: Collection<User>,
    jwt_secret: String,
    email_service: crate::email::EmailService,
}

impl AuthService {
    pub async fn new(mongo_uri: &str, db_name: &str) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(mongo_uri).await?;
        let db = client.database(db_name);
        let users = db.collection::<User>("users");

        // Create unique index on email
        let _ = users
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(doc! { "email": 1 })
                    .options(
                        mongodb::options::IndexOptions::builder()
                            .unique(true)
                            .build(),
                    )
                    .build(),
            )
            .await;

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "epeletii-dev-secret-change-in-prod".to_string());

        let email_service = crate::email::EmailService::new();

        Ok(Self { users, jwt_secret, email_service })
    }

    /// Register a new user.
    pub async fn signup(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<(String, AuthUser), String> {
        let email = email.trim().to_lowercase();
        if !email.contains('@') {
            return Err("Invalid email".to_string());
        }
        if password.len() < 6 {
            return Err("Password must be at least 6 characters".to_string());
        }
        let display_name = display_name.trim();
        if display_name.is_empty() {
            return Err("Display name cannot be empty".to_string());
        }

        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        let user = User {
            id: None,
            email: email.clone(),
            password_hash,
            display_name: display_name.to_string(),
            created_at: Utc::now().timestamp(),
            games_played: 0,
            games_won: 0,
            total_score: 0,
            reset_token: None,
            reset_token_expires: None,
        };

        self.users
            .insert_one(user)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key") {
                    "Email already registered".to_string()
                } else {
                    format!("Database error: {}", e)
                }
            })?;

        // Send welcome email in background
        let email_svc = self.email_service.clone();
        let email_to = email.clone();
        let display_name_to = display_name.to_string();
        tokio::spawn(async move {
            if let Err(e) = email_svc.send_welcome_email(&email_to, &display_name_to).await {
                log::error!("Failed to send welcome email to {}: {}", email_to, e);
            }
        });

        let token = self.create_token(&email, display_name)?;
        Ok((
            token,
            AuthUser {
                email,
                display_name: display_name.to_string(),
            },
        ))
    }

    /// Sign in an existing user.
    pub async fn signin(&self, email: &str, password: &str) -> Result<(String, AuthUser), String> {
        let email = email.trim().to_lowercase();

        let user = self
            .users
            .find_one(doc! { "email": &email })
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Invalid email or password".to_string())?;

        let valid = bcrypt::verify(password, &user.password_hash)
            .map_err(|_| "Invalid email or password".to_string())?;

        if !valid {
            return Err("Invalid email or password".to_string());
        }

        let token = self.create_token(&email, &user.display_name)?;
        Ok((
            token,
            AuthUser {
                email,
                display_name: user.display_name,
            },
        ))
    }

    /// Verify a JWT token and return the claims.
    pub fn verify_token(&self, token: &str) -> Result<Claims, String> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| format!("Invalid token: {}", e))?;
        Ok(token_data.claims)
    }

    fn create_token(&self, email: &str, display_name: &str) -> Result<String, String> {
        let now = Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: email.to_string(),
            name: display_name.to_string(),
            exp: now + 86400 * 7, // 7 days
            iat: now,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| format!("Failed to create token: {}", e))
    }

    /// Record the game result for a player
    pub async fn record_game_result(&self, email: &str, score: u32, won: bool) {
        let update = doc! {
            "$inc": {
                "games_played": 1,
                "games_won": if won { 1 } else { 0 },
                "total_score": score,
            }
        };
        let _ = self.users.update_one(doc! { "email": email }, update).await;
    }

    /// Get top 10 players ranked by total_score
    pub async fn get_leaderboard(&self) -> Result<Vec<crate::protocol::LeaderboardEntry>, String> {
        let filter = doc! {};
        let mut cursor = self.users.find(filter)
            .sort(doc! { "total_score": -1 })
            .limit(10)
            .await
            .map_err(|e| format!("Failed to fetch leaderboard: {}", e))?;
        
        let mut list = Vec::new();
        while cursor.advance().await.unwrap_or(false) {
            if let Ok(user) = cursor.deserialize_current() {
                list.push(crate::protocol::LeaderboardEntry {
                    display_name: user.display_name,
                    games_played: user.games_played,
                    games_won: user.games_won,
                    total_score: user.total_score,
                });
            }
        }
        Ok(list)
    }

    /// Request a password reset: generate token and send email
    pub async fn request_password_reset(&self, email: &str) -> Result<(), String> {
        let email = email.trim().to_lowercase();
        let _user = self
            .users
            .find_one(doc! { "email": &email })
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Email not registered".to_string())?;

        let token = uuid::Uuid::new_v4().to_string();
        let expires = Utc::now().timestamp() + 3600; // 1 hour validity

        let update = doc! {
            "$set": {
                "reset_token": &token,
                "reset_token_expires": expires,
            }
        };

        self.users
            .update_one(doc! { "email": &email }, update)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Send reset email in background
        let email_svc = self.email_service.clone();
        let email_to = email;
        let token_to = token;
        tokio::spawn(async move {
            if let Err(e) = email_svc.send_reset_password_email(&email_to, &token_to).await {
                log::error!("Failed to send password reset email to {}: {}", email_to, e);
            }
        });

        Ok(())
    }

    /// Reset password using token
    pub async fn reset_password(&self, email: &str, token: &str, new_password: &str) -> Result<(), String> {
        let email = email.trim().to_lowercase();
        let token = token.trim();
        if new_password.len() < 6 {
            return Err("Password must be at least 6 characters".to_string());
        }

        let user = self
            .users
            .find_one(doc! { "email": &email })
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "User not found".to_string())?;

        let user_token = user.reset_token.ok_or_else(|| "No reset request pending".to_string())?;
        let expires = user.reset_token_expires.ok_or_else(|| "No reset request pending".to_string())?;

        if user_token != token {
            return Err("Invalid reset token".to_string());
        }

        if Utc::now().timestamp() > expires {
            return Err("Reset token has expired".to_string());
        }

        let password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        let update = doc! {
            "$set": {
                "password_hash": password_hash,
            },
            "$unset": {
                "reset_token": "",
                "reset_token_expires": "",
            }
        };

        self.users
            .update_one(doc! { "email": &email }, update)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(())
    }
}
