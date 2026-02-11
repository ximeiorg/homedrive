use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};

use super::super::entity::{album_files, albums, file_contents, member_files};

pub struct Query;

impl Query {
    /// 根据ID查询单个相册
    pub async fn find_album_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> Result<Option<albums::Model>, sea_orm::DbErr> {
        albums::Entity::find_by_id(id).one(db).await
    }

    /// 根据用户ID查询所有相册
    pub async fn find_albums_by_member_id(
        db: &DatabaseConnection,
        member_id: i64,
    ) -> Result<Vec<albums::Model>, sea_orm::DbErr> {
        albums::Entity::find()
            .filter(albums::Column::MemberId.eq(member_id))
            .order_by_desc(albums::Column::CreatedAt)
            .all(db)
            .await
    }

    /// 根据用户ID和相册名查询相册
    pub async fn find_album_by_member_and_name(
        db: &DatabaseConnection,
        member_id: i64,
        name: &str,
    ) -> Result<Option<albums::Model>, sea_orm::DbErr> {
        albums::Entity::find()
            .filter(albums::Column::MemberId.eq(member_id))
            .filter(albums::Column::Name.eq(name))
            .one(db)
            .await
    }

    /// 检查相册名称是否已存在（同一用户下）
    pub async fn album_name_exists(
        db: &DatabaseConnection,
        member_id: i64,
        name: &str,
    ) -> Result<bool, sea_orm::DbErr> {
        let count = albums::Entity::find()
            .filter(albums::Column::MemberId.eq(member_id))
            .filter(albums::Column::Name.eq(name))
            .count(db)
            .await?;

        Ok(count > 0)
    }

    /// 分页查询用户的相册
    pub async fn list_albums_by_member(
        db: &DatabaseConnection,
        member_id: i64,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<albums::Model>, u64), sea_orm::DbErr> {
        let select = albums::Entity::find()
            .filter(albums::Column::MemberId.eq(member_id))
            .order_by_desc(albums::Column::CreatedAt);

        let total = select.clone().count(db).await?;
        let paginator = select.paginate(db, page_size);
        let results = paginator.fetch_page(page - 1).await?;

        Ok((results, total))
    }

    /// 查询相册中的文件列表（包含 file_contents 信息）
    pub async fn list_files_in_album(
        db: &DatabaseConnection,
        album_id: i64,
        page: u64,
        page_size: u64,
    ) -> Result<
        (
            Vec<(
                album_files::Model,
                member_files::Model,
                Option<file_contents::Model>,
            )>,
            u64,
        ),
        sea_orm::DbErr,
    > {
        // 先查询 album_files
        let select = album_files::Entity::find()
            .filter(album_files::Column::AlbumId.eq(album_id))
            .order_by_desc(album_files::Column::CreatedAt);

        let total = select.clone().count(db).await?;
        let paginator = select.paginate(db, page_size);
        let album_file_results = paginator.fetch_page(page - 1).await?;

        // 加载关联的 member_files 和 file_contents
        let mut results = Vec::new();
        for album_file in album_file_results {
            // 查询 member_file
            let member_file = member_files::Entity::find_by_id(album_file.member_file_id)
                .one(db)
                .await?;

            if let Some(mf) = member_file {
                // 查询 file_content
                let file_content = file_contents::Entity::find_by_id(mf.file_content_id)
                    .one(db)
                    .await?;

                results.push((album_file, mf, file_content));
            }
        }

        Ok((results, total))
    }

    /// 检查文件是否已在相册中
    pub async fn file_in_album(
        db: &DatabaseConnection,
        album_id: i64,
        member_file_id: i64,
    ) -> Result<bool, sea_orm::DbErr> {
        let count = album_files::Entity::find()
            .filter(album_files::Column::AlbumId.eq(album_id))
            .filter(album_files::Column::MemberFileId.eq(member_file_id))
            .count(db)
            .await?;

        Ok(count > 0)
    }

    /// 查询相册中的文件数量
    pub async fn count_files_in_album(
        db: &DatabaseConnection,
        album_id: i64,
    ) -> Result<u64, sea_orm::DbErr> {
        album_files::Entity::find()
            .filter(album_files::Column::AlbumId.eq(album_id))
            .count(db)
            .await
    }

    /// 查询文件所属的所有相册
    pub async fn find_albums_by_member_file(
        db: &DatabaseConnection,
        member_file_id: i64,
    ) -> Result<Vec<albums::Model>, sea_orm::DbErr> {
        // 先查询 album_files 获取 album_id 列表
        let album_file_records = album_files::Entity::find()
            .filter(album_files::Column::MemberFileId.eq(member_file_id))
            .all(db)
            .await?;

        let album_ids: Vec<i64> = album_file_records
            .into_iter()
            .map(|af| af.album_id)
            .collect();

        if album_ids.is_empty() {
            return Ok(Vec::new());
        }

        albums::Entity::find()
            .filter(albums::Column::Id.is_in(album_ids))
            .all(db)
            .await
    }
}
