use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::{Deserialize, Serialize};

use super::super::entity::member_files;

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberFileQuery {
    pub id: Option<i64>,
    pub member_id: Option<i64>,
    pub file_content_id: Option<i64>,
    pub file_name: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

pub struct Query;

impl Query {
    /// 根据ID查询单个成员文件记录
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> Result<Option<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find_by_id(id).one(db).await
    }

    /// 根据成员ID查询文件记录
    pub async fn find_by_member_id(
        db: &DatabaseConnection,
        member_id: i64,
    ) -> Result<Vec<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find()
            .filter(member_files::Column::MemberId.eq(member_id))
            .all(db)
            .await
    }

    /// 根据文件内容ID查询文件记录
    pub async fn find_by_file_content_id(
        db: &DatabaseConnection,
        file_content_id: i64,
    ) -> Result<Vec<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find()
            .filter(member_files::Column::FileContentId.eq(file_content_id))
            .all(db)
            .await
    }

    /// 根据文件名查询文件记录
    pub async fn find_by_file_name(
        db: &DatabaseConnection,
        file_name: &str,
    ) -> Result<Vec<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find()
            .filter(member_files::Column::FileName.eq(file_name))
            .all(db)
            .await
    }

    /// 查询所有成员文件记录（分页）
    pub async fn find_all(
        db: &DatabaseConnection,
        query: MemberFileQuery,
    ) -> Result<(Vec<member_files::Model>, u64), sea_orm::DbErr> {
        let mut select = member_files::Entity::find();

        // 应用过滤条件
        if let Some(id) = query.id {
            select = select.filter(member_files::Column::Id.eq(id));
        }
        if let Some(member_id) = query.member_id {
            select = select.filter(member_files::Column::MemberId.eq(member_id));
        }
        if let Some(file_content_id) = query.file_content_id {
            select = select.filter(member_files::Column::FileContentId.eq(file_content_id));
        }
        if let Some(ref file_name) = query.file_name {
            select = select.filter(member_files::Column::FileName.eq(file_name));
        }

        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);

        // 获取总数
        let total = select.clone().count(db).await?;

        // 分页查询
        let paginator = select.paginate(db, page_size);
        let results = paginator.fetch_page(page - 1).await?;

        Ok((results, total))
    }

    /// 检查成员和文件内容的关联是否存在
    pub async fn association_exists(
        db: &DatabaseConnection,
        member_id: i64,
        file_content_id: i64,
    ) -> Result<bool, sea_orm::DbErr> {
        let count = member_files::Entity::find()
            .filter(member_files::Column::MemberId.eq(member_id))
            .filter(member_files::Column::FileContentId.eq(file_content_id))
            .count(db)
            .await?;

        Ok(count > 0)
    }

    /// 根据成员ID和文件名查询文件记录
    pub async fn find_by_member_and_name(
        db: &DatabaseConnection,
        member_id: i64,
        file_name: &str,
    ) -> Result<Option<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find()
            .filter(member_files::Column::MemberId.eq(member_id))
            .filter(member_files::Column::FileName.eq(file_name))
            .one(db)
            .await
    }

    /// 获取成员的最新文件
    pub async fn find_recent_files_by_member(
        db: &DatabaseConnection,
        member_id: i64,
        limit: u64,
    ) -> Result<Vec<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find()
            .filter(member_files::Column::MemberId.eq(member_id))
            .order_by_desc(member_files::Column::CreatedAt)
            .limit(limit)
            .all(db)
            .await
    }

    /// 根据描述模糊搜索文件
    pub async fn search_by_description(
        db: &DatabaseConnection,
        keyword: &str,
    ) -> Result<Vec<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find()
            .filter(member_files::Column::Description.like(format!("%{}%", keyword)))
            .all(db)
            .await
    }
}
