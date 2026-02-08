use crate::error::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use schema::member::{
    CreateMemberRequest, InitAdminRequest, InitAdminResponse, IsEmptyResponse, LoginRequest,
    LoginResponse, MemberListResponse, MemberResponse, UpdateMemberRequest,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use store::DatabaseConnection;

/// 默认存储空间（10GB）
const DEFAULT_STORAGE_TOTAL: i64 = 10 * 1024 * 1024 * 1024;

/// 全局 JWT 密钥
static JWT_SECRET: OnceCell<String> = OnceCell::new();

/// 初始化 JWT 密钥（在服务启动时调用）
pub fn init_jwt_secret(secret: String) {
    JWT_SECRET.set(secret).ok();
}

/// 获取 JWT 密钥
pub fn get_jwt_secret() -> String {
    JWT_SECRET.get().cloned().unwrap_or_else(|| {
        // 默认密钥，仅用于开发环境
        "default-secret-key-for-development".to_string()
    })
}

pub struct MemberService;

impl MemberService {
    /// 获取成员的存储使用量
    async fn get_storage_used(db: &DatabaseConnection, member_id: i64) -> Result<i64> {
        // 通过member_files关联file_contents计算存储使用量
        let member_files = store::entity::member_files::Entity::find()
            .filter(store::entity::member_files::Column::MemberId.eq(member_id))
            .find_with_related(store::entity::file_contents::Entity)
            .all(db)
            .await?;

        let total_size: i64 = member_files
            .into_iter()
            .flat_map(|(_, file_contents)| file_contents)
            .map(|fc| fc.file_size)
            .sum();

        Ok(total_size)
    }

    /// 确定成员状态（基于最后活跃时间）
    fn determine_status(last_active: Option<DateTime<Utc>>) -> String {
        match last_active {
            Some(last) => {
                let now = Utc::now();
                let duration = now.signed_duration_since(last);
                if duration.num_minutes() < 5 {
                    "online".to_string()
                } else if duration.num_minutes() < 30 {
                    "away".to_string()
                } else {
                    "offline".to_string()
                }
            }
            None => "offline".to_string(),
        }
    }

    /// 构建完整的成员响应（包含存储信息）
    async fn build_member_response(
        db: &DatabaseConnection,
        member: store::entity::members::Model,
    ) -> Result<MemberResponse> {
        let storage_used = Self::get_storage_used(db, member.id).await.unwrap_or(0);
        let last_active: Option<DateTime<Utc>> = None; // TODO: 实现最后活跃时间追踪
        let status = Self::determine_status(last_active);

        Ok(MemberResponse {
            id: member.id,
            username: member.username,
            avatar: member.avatar,
            storage_tag: member.storage_tag,
            storage_used,
            storage_total: DEFAULT_STORAGE_TOTAL,
            last_active,
            status,
            created_at: member.created_at,
        })
    }

    /// 创建新成员
    pub async fn create_member(
        db: &DatabaseConnection,
        data: CreateMemberRequest,
    ) -> Result<MemberResponse> {
        // 对密码进行 bcrypt 哈希
        let hashed_password = bcrypt::hash(&data.password, bcrypt::DEFAULT_COST).map_err(|e| {
            tracing::error!("Failed to hash password: {:?}", e);
            crate::error::ServiceError::Unknown
        })?;

        let create_data = store::member::mutation::CreateMember {
            username: data.username,
            password: hashed_password,
            avatar: data.avatar,
            storage_tag: data.storage_tag,
        };

        let member = store::member::mutation::Mutation::create(db, create_data).await?;

        Self::build_member_response(db, member).await
    }

    /// 获取成员详情
    pub async fn get_member(db: &DatabaseConnection, id: i64) -> Result<Option<MemberResponse>> {
        let member: Option<store::entity::members::Model> =
            store::member::query::Query::find_by_id(db, id).await?;

        match member {
            Some(m) => {
                let response = Self::build_member_response(db, m).await?;
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    /// 更新成员信息
    pub async fn update_member(
        db: &DatabaseConnection,
        id: i64,
        data: UpdateMemberRequest,
    ) -> Result<MemberResponse> {
        let update_data = store::member::mutation::UpdateMember {
            username: data.username,
            password: data.password,
            avatar: data.avatar,
            storage_tag: data.storage_tag,
        };

        let member = store::member::mutation::Mutation::update(db, id, update_data).await?;

        Self::build_member_response(db, member).await
    }

    /// 删除成员
    pub async fn delete_member(db: &DatabaseConnection, id: i64) -> Result<()> {
        store::member::mutation::Mutation::delete(db, id).await?;
        Ok(())
    }

    /// 获取成员列表（分页）
    pub async fn list_members(
        db: &DatabaseConnection,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<MemberListResponse> {
        let query = store::member::query::MemberQuery {
            id: None,
            username: None,
            storage_tag: None,
            page,
            page_size,
        };

        let (members, total): (Vec<store::entity::members::Model>, u64) =
            store::member::query::Query::find_all(db, query).await?;

        let mut member_responses: Vec<MemberResponse> = Vec::new();
        for m in members {
            match Self::build_member_response(db, m).await {
                Ok(response) => member_responses.push(response),
                Err(e) => tracing::error!("Failed to build member response: {:?}", e),
            }
        }

        Ok(MemberListResponse {
            members: member_responses,
            total,
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(20),
        })
    }

    /// 根据用户名查询成员
    pub async fn get_member_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<Option<MemberResponse>> {
        let member: Option<store::entity::members::Model> =
            store::member::query::Query::find_by_username(db, username).await?;

        match member {
            Some(m) => {
                let response = Self::build_member_response(db, m).await?;
                Ok(Some(response))
            }
            None => Ok(None),
        }
    }

    /// 检查用户名是否存在
    pub async fn username_exists(db: &DatabaseConnection, username: &str) -> Result<bool> {
        let exists: bool = store::member::query::Query::username_exists(db, username).await?;
        Ok(exists)
    }

    /// 更新成员头像
    pub async fn update_avatar(
        db: &DatabaseConnection,
        id: i64,
        avatar: String,
    ) -> Result<MemberResponse> {
        let member = store::member::mutation::Mutation::update_avatar(db, id, avatar).await?;

        Self::build_member_response(db, member).await
    }

    /// 更新密码
    pub async fn update_password(
        db: &DatabaseConnection,
        id: i64,
        new_password: String,
    ) -> Result<MemberResponse> {
        //对新密码进行 bcrypt 哈希
        let hashed_password = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST).map_err(|e| {
            tracing::error!("Failed to hash password: {:?}", e);
            crate::error::ServiceError::Unknown
        })?;

        let member =
            store::member::mutation::Mutation::update_password(db, id, hashed_password).await?;

        Self::build_member_response(db, member).await
    }

    /// 登录验证
    pub async fn login(db: &DatabaseConnection, data: LoginRequest) -> Result<LoginResponse> {
        // 根据用户名查找成员
        let member = store::member::query::Query::find_by_username(db, &data.username)
            .await?
            .ok_or(crate::error::ServiceError::InvalidCredentials)?;

        // 使用 bcrypt 验证密码
        let password_matches = bcrypt::verify(&data.password, &member.password).map_err(|e| {
            tracing::error!("Failed to verify password: {:?}", e);
            crate::error::ServiceError::InvalidCredentials
        })?;

        if !password_matches {
            return Err(crate::error::ServiceError::InvalidCredentials);
        }

        // 生成 token
        let token = generate_token(&member);

        // 构建完整的成员响应
        let member_response = Self::build_member_response(db, member).await?;

        Ok(LoginResponse {
            token,
            member: member_response,
        })
    }

    /// 检查 member 表是否为空
    pub async fn is_empty(db: &DatabaseConnection) -> Result<IsEmptyResponse> {
        let is_empty = store::member::query::Query::is_empty(db).await?;
        Ok(IsEmptyResponse { is_empty })
    }

    /// 初始化管理员（仅当 member 表为空时有效）
    pub async fn init_admin(
        db: &DatabaseConnection,
        data: InitAdminRequest,
    ) -> Result<InitAdminResponse> {
        // 检查是否已经有成员
        let is_empty = store::member::query::Query::is_empty(db).await?;
        if !is_empty {
            return Ok(InitAdminResponse {
                success: false,
                message: "You can't do this action".to_string(),
                member: None,
            });
        }

        // 验证输入
        if data.username.is_empty() {
            return Ok(InitAdminResponse {
                success: false,
                message: "Username cannot be empty".to_string(),
                member: None,
            });
        }
        if data.password.len() < 6 {
            return Ok(InitAdminResponse {
                success: false,
                message: "Password must be at least 6 characters".to_string(),
                member: None,
            });
        }
        if data.storage_tag.is_empty() {
            return Ok(InitAdminResponse {
                success: false,
                message: "Storage tag cannot be empty".to_string(),
                member: None,
            });
        }
        if data.storage_tag.len() > 50 {
            return Ok(InitAdminResponse {
                success: false,
                message: "Storage tag must be less than 50 characters".to_string(),
                member: None,
            });
        }
        // 检查 storage_tag 是否已存在
        if store::member::query::Query::storage_tag_exists(db, &data.storage_tag).await? {
            return Ok(InitAdminResponse {
                success: false,
                message: "Storage tag already exists".to_string(),
                member: None,
            });
        }

        // 使用用户提供的 storage_tag
        let storage_tag = data.storage_tag;

        // 调用 create_member 来创建管理员（会进行密码哈希和验证）
        let create_request = CreateMemberRequest {
            username: data.username,
            password: data.password,
            avatar: None,
            storage_tag,
        };

        let member = Self::create_member(db, create_request).await?;

        Ok(InitAdminResponse {
            success: true,
            message: "Admin user created successfully".to_string(),
            member: Some(member),
        })
    }
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i64,
    aud: Option<String>,
    exp: u64,
    iat: u64,
}

/// 生成 JWT token
fn generate_token(member: &store::entity::members::Model) -> String {
    let secret = get_jwt_secret();
    let header = jsonwebtoken::Header::default();
    let now = Utc::now();
    let expiration = now + chrono::Duration::hours(24);

    let claims = Claims {
        sub: member.id,
        aud: Some("homedrive".to_string()),
        exp: expiration.timestamp() as u64,
        iat: now.timestamp() as u64,
    };

    jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap_or_default()
}
