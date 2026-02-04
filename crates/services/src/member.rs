use crate::error::Result;
use schema::member::{
    CreateMemberRequest, MemberListResponse, MemberResponse, UpdateMemberRequest,
};
use store::DatabaseConnection;

pub struct MemberService;

impl MemberService {
    /// 创建新成员
    pub async fn create_member(
        db: &DatabaseConnection,
        data: CreateMemberRequest,
    ) -> Result<MemberResponse> {
        let create_data = store::member::mutation::CreateMember {
            username: data.username,
            password: data.password,
            avatar: data.avatar,
            storage_tag: data.storage_tag,
        };

        let member = store::member::mutation::Mutation::create(db, create_data).await?;

        Ok(MemberResponse {
            id: member.id,
            username: member.username,
            avatar: member.avatar,
            storage_tag: member.storage_tag,
            created_at: member.created_at,
        })
    }

    /// 获取成员详情
    pub async fn get_member(db: &DatabaseConnection, id: i64) -> Result<Option<MemberResponse>> {
        let member: Option<store::entity::members::Model> =
            store::member::query::Query::find_by_id(db, id).await?;

        Ok(member.map(|m| MemberResponse {
            id: m.id,
            username: m.username,
            avatar: m.avatar,
            storage_tag: m.storage_tag,
            created_at: m.created_at,
        }))
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

        Ok(MemberResponse {
            id: member.id,
            username: member.username,
            avatar: member.avatar,
            storage_tag: member.storage_tag,
            created_at: member.created_at,
        })
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

        let member_responses: Vec<MemberResponse> = members
            .into_iter()
            .map(|m| MemberResponse {
                id: m.id,
                username: m.username,
                avatar: m.avatar,
                storage_tag: m.storage_tag,
                created_at: m.created_at,
            })
            .collect();

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

        Ok(member.map(|m| MemberResponse {
            id: m.id,
            username: m.username,
            avatar: m.avatar,
            storage_tag: m.storage_tag,
            created_at: m.created_at,
        }))
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

        Ok(MemberResponse {
            id: member.id,
            username: member.username,
            avatar: member.avatar,
            storage_tag: member.storage_tag,
            created_at: member.created_at,
        })
    }

    /// 更新密码
    pub async fn update_password(
        db: &DatabaseConnection,
        id: i64,
        new_password: String,
    ) -> Result<MemberResponse> {
        let member =
            store::member::mutation::Mutation::update_password(db, id, new_password).await?;

        Ok(MemberResponse {
            id: member.id,
            username: member.username,
            avatar: member.avatar,
            storage_tag: member.storage_tag,
            created_at: member.created_at,
        })
    }
}
