use std::str::FromStr;

use async_trait::async_trait;

use derive_builder::Builder;
use sqlx::{Row};
use uuid::Uuid;

use crate::{backend::engine::SDKEngine, common::commons::SortOrder, errors::sdk::SDKError};

use super::member::{Member, MemberRole};

#[async_trait]
pub trait MemberCrudOperations {
    async fn create_member(&self, input: CreateMemberInput) -> Result<Member, SDKError>;
    async fn get_member(&self, id: Uuid) -> Result<Member, SDKError>;
    async fn get_members(&self, input: GetMembersInput) -> Result<Vec<Member>, SDKError>;
    async fn update_member(&self, id: Uuid, input: UpdateMemberInput) -> Result<Member, SDKError>;
    async fn delete_member(&self, id: Uuid) -> Result<Member, SDKError>;
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct CreateMemberInput {
    pub name: String,
    pub email: String,
    pub role: MemberRole,
    pub github_id: Option<String>,
    pub google_id: Option<String>,
    pub photo_url: Option<String>,
    pub password_hash: Option<String>,
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct UpdateMemberInput {
    #[builder(setter(strip_option), default)]
    pub name: Option<String>,
    #[builder(setter(strip_option), default)]
    pub email: Option<String>,
    #[builder(setter(strip_option), default)]
    pub role: Option<MemberRole>,
    #[builder(setter(strip_option), default)]
    pub github_id: Option<String>,
    #[builder(setter(strip_option), default)]
    pub google_id: Option<String>,
    #[builder(setter(strip_option), default)]
    pub photo_url: Option<String>,
    #[builder(setter(strip_option), default)]
    pub password_hash: Option<String>,
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct GetMembersInput {
    #[builder(setter(strip_option), default)]
    pub filter: Option<GetMembersWhere>,

    #[builder(setter(strip_option), default)]
    pub sort_by: Option<String>,
    #[builder(setter(strip_option), default)]
    pub sort_order: Option<SortOrder>,

    #[builder(setter(into, strip_option), default = "Some(100)")]
    pub limit: Option<i32>,
    #[builder(setter(into, strip_option), default = "Some(0)")]
    pub offset: Option<i32>,
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct GetMembersWhere {
    #[builder(setter(strip_option), default)]
    pub name: Option<String>,
    #[builder(setter(strip_option), default)]
    pub email: Option<String>,
    #[builder(setter(strip_option), default)]
    pub role: Option<MemberRole>,
    #[builder(setter(strip_option), default)]
    pub github_id: Option<String>,
    #[builder(setter(strip_option), default)]
    pub google_id: Option<String>,
    #[builder(setter(strip_option), default)]
    pub photo_url: Option<String>,

    #[builder(setter(strip_option), default)]
    pub _and: Option<Vec<GetMembersWhere>>,
    #[builder(setter(strip_option), default)]
    pub _or: Option<Vec<GetMembersWhere>>,
}

impl GetMembersWhere {
    pub fn compile_sql(&self) -> String {
        let mut where_clause = String::new();
        let mut and_clauses = Vec::new();
        let mut or_clauses = Vec::new();

        if let Some(name) = &self.name {
            and_clauses.push(format!("name = '{}'", name));
        }
        if let Some(email) = &self.email {
            and_clauses.push(format!("email = '{}'", email));
        }
        if let Some(role) = &self.role {
            and_clauses.push(format!("role = '{}'", role));
        }
        if let Some(github_id) = &self.github_id {
            and_clauses.push(format!("github_id = '{}'", github_id));
        }
        if let Some(google_id) = &self.google_id {
            and_clauses.push(format!("google_id = '{}'", google_id));
        }
        if let Some(photo_url) = &self.photo_url {
            and_clauses.push(format!("photo_url = '{}'", photo_url));
        }

        if let Some(and) = &self._and {
            for and_clause in and {
                and_clauses.push(and_clause.compile_sql());
            }
        }
        if let Some(or) = &self._or {
            for or_clause in or {
                or_clauses.push(or_clause.compile_sql());
            }
        }

        if !and_clauses.is_empty() {
            where_clause.push_str(&format!("({})", and_clauses.join(" AND ")));
        }
        if !or_clauses.is_empty() {
            if !where_clause.is_empty() {
                where_clause.push_str(" OR ");
            }
            where_clause.push_str(&format!("({})", or_clauses.join(" OR ")));
        }

        where_clause
    }
}

#[async_trait]
impl MemberCrudOperations for SDKEngine {
    async fn create_member(&self, input: CreateMemberInput) -> Result<Member, SDKError> {
        let member_final_info = sqlx::query!(
            r#"
            INSERT INTO members (name, email, role, github_id, google_id, photo_url, password_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            input.name,
            input.email,
            input.role.to_string(),
            input.github_id,
            input.google_id,
            input.photo_url,
            input.password_hash
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        let member = Member {
            id: member_final_info.id,
            created_at: member_final_info.created_at,
            updated_at: member_final_info.updated_at,
            name: member_final_info.name,
            email: member_final_info.email,
            role: member_final_info
                .role
                .and_then(|a| MemberRole::from_str(&a).ok())
                .unwrap_or_default(),
            github_id: member_final_info.github_id,
            google_id: member_final_info.google_id,
            photo_url: member_final_info.photo_url,
            password_hash: member_final_info.password_hash,
        };

        Ok(member)
    }

    async fn get_member(&self, id: Uuid) -> Result<Member, SDKError> {
        let member_info = sqlx::query!(
            r#"
            SELECT id, created_at, updated_at, name, email, role, github_id, google_id, photo_url, password_hash
            FROM members
            WHERE id = $1
            "#,
            id
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        let member = Member {
            id: member_info.id,
            created_at: member_info.created_at,
            updated_at: member_info.updated_at,
            name: member_info.name,
            email: member_info.email,
            role: member_info
                .role
                .and_then(|a| MemberRole::from_str(&a).ok())
                .unwrap_or_default(),
            github_id: member_info.github_id,
            google_id: member_info.google_id,
            photo_url: member_info.photo_url,
            password_hash: member_info.password_hash,
        };

        Ok(member)
    }

    async fn get_members(&self, input: GetMembersInput) -> Result<Vec<Member>, SDKError> {
        let mut query = "SELECT * FROM members ".to_string();

        if let Some(filter) = input.filter {
            query.push_str(format!("WHERE {} ", filter.compile_sql()).as_str());
        }

        if let Some(sort_by) = input.sort_by {
            query.push_str(format!("ORDER BY {} ", sort_by).as_str());
        }

        if let Some(sort_order) = input.sort_order {
            query.push_str(format!("{} ", sort_order).as_str());
        }

        if let Some(limit) = input.limit {
            query.push_str(format!("LIMIT {} ", limit).as_str());
        }

        if let Some(offset) = input.offset {
            query.push_str(format!("OFFSET {} ", offset).as_str());
        }

        let members_info = sqlx::query(query.as_str())
            .fetch_all(self.pool.as_ref())
            .await?;

        let members = members_info
            .iter()
            .map(|x| Member {
                id: x.get("id"),
                created_at: x.get("created_at"),
                updated_at: x.get("updated_at"),
                name: x.get("name"),
                email: x.get("email"),
                role: x
                    .get::<'_, Option<String>, _>("status")
                    .and_then(|a| MemberRole::from_str(&a).ok())
                    .unwrap_or_default(),
                github_id: x.get("github_id"),
                google_id: x.get("google_id"),
                photo_url: x.get("photo_url"),
                password_hash: x.get("password_hash"),
            })
            .collect::<Vec<Member>>();

        Ok(members)
    }

    async fn update_member(&self, id: Uuid, input: UpdateMemberInput) -> Result<Member, SDKError> {
        let member_final_info = sqlx::query!(
            r#"
            UPDATE members
            SET
                name = COALESCE($1, name),
                email = COALESCE($2, email),
                role = COALESCE($3, role),
                github_id = COALESCE($4, github_id),
                google_id = COALESCE($5, google_id),
                photo_url = COALESCE($6, photo_url),
                password_hash = COALESCE($7, password_hash)
            WHERE id = $8
            RETURNING *
            "#,
            input.name,
            input.email,
            input.role.map(|role| role.to_string()),
            input.github_id,
            input.google_id,
            input.photo_url,
            input.password_hash,
            id
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        let member = Member {
            id: member_final_info.id,
            created_at: member_final_info.created_at,
            updated_at: member_final_info.updated_at,
            name: member_final_info.name,
            email: member_final_info.email,
            role: member_final_info
                .role
                .and_then(|a| MemberRole::from_str(&a).ok())
                .unwrap_or_default(),
            github_id: member_final_info.github_id,
            google_id: member_final_info.google_id,
            photo_url: member_final_info.photo_url,
            password_hash: member_final_info.password_hash,
        };

        Ok(member)
    }

    async fn delete_member(&self, id: Uuid) -> Result<Member, SDKError> {
        let member_info = sqlx::query!(
            r#"
            DELETE FROM members WHERE id = $1
            RETURNING *
            "#,
            id
        )
        .fetch_one(self.pool.as_ref())
        .await?;

        let member = Member {
            id: member_info.id,
            created_at: member_info.created_at,
            updated_at: member_info.updated_at,
            name: member_info.name,
            email: member_info.email,
            role: member_info
                .role
                .and_then(|a| MemberRole::from_str(&a).ok())
                .unwrap_or_default(),
            github_id: member_info.github_id,
            google_id: member_info.google_id,
            photo_url: member_info.photo_url,
            password_hash: member_info.password_hash,
        };

        Ok(member)
    }
}
