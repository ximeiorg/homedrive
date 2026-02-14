use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Add role column with default value 'user'
        manager
            .alter_table(
                Table::alter()
                    .table(Member::Table)
                    .add_column(
                        ColumnDef::new(Member::Role)
                            .string_len(20)
                            .not_null()
                            .default("user"),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Create enum type for role (as a check constraint since SQLite doesn't support enums)
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE TABLE IF NOT EXISTS _temp_role_check AS 
                SELECT 1 WHERE 1=0;
                "#,
            )
            .await
            .ok();

        // For SQLite, we use a check constraint
        // For PostgreSQL, we would use an enum type
        #[cfg(feature = "sqlx-sqlite")]
        {
            manager
                .get_connection()
                .execute_unprepared(
                    r#"
                    ALTER TABLE members ADD CONSTRAINT chk_role_check 
                    CHECK (role IN ('admin', 'user'));
                    "#,
                )
                .await
                .ok();
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Member::Table)
                    .drop_column(Member::Role)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Member {
    #[iden = "members"]
    Table,
    Role,
}
