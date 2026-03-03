use sea_orm::sea_query::Alias;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};

use super::super::entity::{file_contents, member_files};

/// 排序字段枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SortField {
    CreatedAt,
    FileName,
    FileSize,
    DeletedAt,
}

/// 排序方向
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 文件类型过滤器
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileTypeFilter {
    Image,
    Video,
    Audio,
    Document,
    Archive,
    Other,
}

impl FileTypeFilter {
    /// 获取对应的 MIME 类型前缀
    pub fn get_mime_prefixes(&self) -> Vec<&'static str> {
        match self {
            FileTypeFilter::Image => vec!["image/"],
            FileTypeFilter::Video => vec!["video/"],
            FileTypeFilter::Audio => vec!["audio/"],
            FileTypeFilter::Document => vec![
                "text/",
                "application/pdf",
                "application/msword",
                "application/vnd.openxmlformats-officedocument",
            ],
            FileTypeFilter::Archive => vec![
                "application/zip",
                "application/x-rar-compressed",
                "application/x-7z-compressed",
                "application/gzip",
            ],
            FileTypeFilter::Other => vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberFileQuery {
    pub id: Option<i64>,
    pub member_id: Option<i64>,
    pub file_content_id: Option<i64>,
    pub file_name: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// 列出用户文件的查询参数
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListMemberFilesQuery {
    /// 页码，从 1 开始
    pub page: Option<u64>,
    /// 每页大小，默认 100
    pub page_size: Option<u64>,
    /// 排序字段
    pub sort_by: Option<SortField>,
    /// 排序方向
    pub sort_order: Option<SortOrder>,
    /// 文件类型过滤
    pub file_type: Option<FileTypeFilter>,
    /// 文件名搜索（模糊匹配）
    pub search: Option<String>,
    /// 是否包含已删除文件（回收站）
    pub include_deleted: Option<bool>,
}

/// 回收站查询参数
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListTrashQuery {
    /// 页码，从 1 开始
    pub page: Option<u64>,
    /// 每页大小，默认 100
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

    /// 根据成员ID查询文件记录（排除已删除）
    pub async fn find_by_member_id(
        db: &DatabaseConnection,
        member_id: i64,
    ) -> Result<Vec<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find()
            .filter(member_files::Column::MemberId.eq(member_id))
            .filter(member_files::Column::DeletedAt.is_null())
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
            .filter(member_files::Column::DeletedAt.is_null())
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
            .filter(member_files::Column::DeletedAt.is_null())
            .one(db)
            .await
    }

    /// 获取成员的最新文件（排除已删除）
    pub async fn find_recent_files_by_member(
        db: &DatabaseConnection,
        member_id: i64,
        limit: u64,
    ) -> Result<Vec<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find()
            .filter(member_files::Column::MemberId.eq(member_id))
            .filter(member_files::Column::DeletedAt.is_null())
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
            .filter(member_files::Column::Description.like(format!("%{keyword}%")))
            .filter(member_files::Column::DeletedAt.is_null())
            .all(db)
            .await
    }

    /// 列出用户文件（支持翻页、排序、类型过滤）
    /// 返回包含 file_contents 信息的元组 (member_files, file_contents)
    pub async fn list_files_by_member(
        db: &DatabaseConnection,
        member_id: i64,
        query: ListMemberFilesQuery,
    ) -> Result<
        (
            Vec<(member_files::Model, Option<file_contents::Model>)>,
            u64,
        ),
        sea_orm::DbErr,
    > {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(100);
        let sort_by = query.sort_by.unwrap_or(SortField::CreatedAt);
        let sort_order = query.sort_order.unwrap_or(SortOrder::Desc);

        // 构建基础查询
        let mut select =
            member_files::Entity::find().filter(member_files::Column::MemberId.eq(member_id));

        // 默认排除已删除文件，除非明确要求包含
        if query.include_deleted != Some(true) {
            select = select.filter(member_files::Column::DeletedAt.is_null());
        }

        // 应用文件类型过滤
        if let Some(ref file_type) = query.file_type {
            let mime_prefixes = file_type.get_mime_prefixes();
            if !mime_prefixes.is_empty() {
                let mime_pattern = format!("{}%", mime_prefixes[0]);

                // 通过关联查询 file_contents 的 mime_type
                // 使用条件表达式来过滤
                let mime_filter = sea_orm::Condition::all()
                    .add(file_contents::Column::MimeType.like(mime_pattern.clone()));

                select = select.filter(
                    member_files::Column::FileContentId.in_subquery(
                        sea_orm::sea_query::Query::select()
                            .column(file_contents::Column::Id)
                            .from(Alias::new("file_contents"))
                            .cond_where(mime_filter)
                            .to_owned(),
                    ),
                );
            }
        }

        // 应用文件名搜索
        if let Some(ref search) = query.search {
            select = select.filter(member_files::Column::FileName.like(format!("%{search}%")));
        }

        // 应用排序
        match sort_by {
            SortField::CreatedAt => match sort_order {
                SortOrder::Asc => select = select.order_by_asc(member_files::Column::CreatedAt),
                SortOrder::Desc => select = select.order_by_desc(member_files::Column::CreatedAt),
            },
            SortField::FileName => match sort_order {
                SortOrder::Asc => select = select.order_by_asc(member_files::Column::FileName),
                SortOrder::Desc => select = select.order_by_desc(member_files::Column::FileName),
            },
            SortField::FileSize => match sort_order {
                SortOrder::Asc => select = select.order_by_asc(member_files::Column::CreatedAt),
                SortOrder::Desc => select = select.order_by_desc(member_files::Column::CreatedAt),
            },
            SortField::DeletedAt => match sort_order {
                SortOrder::Asc => select = select.order_by_asc(member_files::Column::DeletedAt),
                SortOrder::Desc => select = select.order_by_desc(member_files::Column::DeletedAt),
            },
        }

        // 获取总数
        let total = select.clone().count(db).await?;

        // 分页查询
        let paginator = select.paginate(db, page_size);
        let results = paginator.fetch_page(page - 1).await?;

        // 加载关联的 file_contents 信息
        let mut results_with_content = Vec::new();
        for member_file in results {
            let file_content = member_file
                .find_related(file_contents::Entity)
                .one(db)
                .await?;
            results_with_content.push((member_file, file_content));
        }

        Ok((results_with_content, total))
    }

    /// 列出回收站文件（已删除文件）
    pub async fn list_trash_by_member(
        db: &DatabaseConnection,
        member_id: i64,
        query: ListTrashQuery,
    ) -> Result<
        (
            Vec<(member_files::Model, Option<file_contents::Model>)>,
            u64,
        ),
        sea_orm::DbErr,
    > {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(100);

        // 构建查询：只查询已删除文件
        let select = member_files::Entity::find()
            .filter(member_files::Column::MemberId.eq(member_id))
            .filter(member_files::Column::DeletedAt.is_not_null())
            .order_by_desc(member_files::Column::DeletedAt);

        // 获取总数
        let total = select.clone().count(db).await?;

        // 分页查询
        let paginator = select.paginate(db, page_size);
        let results = paginator.fetch_page(page - 1).await?;

        // 加载关联的 file_contents 信息
        let mut results_with_content = Vec::new();
        for member_file in results {
            let file_content = member_file
                .find_related(file_contents::Entity)
                .one(db)
                .await?;
            results_with_content.push((member_file, file_content));
        }

        Ok((results_with_content, total))
    }

    /// 根据ID列表查询成员文件记录（验证所有权）
    pub async fn find_by_ids_and_member(
        db: &DatabaseConnection,
        ids: Vec<i64>,
        member_id: i64,
    ) -> Result<Vec<member_files::Model>, sea_orm::DbErr> {
        member_files::Entity::find()
            .filter(member_files::Column::Id.is_in(ids))
            .filter(member_files::Column::MemberId.eq(member_id))
            .all(db)
            .await
    }
}
