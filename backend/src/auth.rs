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

        Ok(Self { users, jwt_secret })
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
}
