use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 기존 테이블이 있으면 삭제
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("error_logs"))
                    .if_exists()
                    .to_owned()
            )
            .await?;

        // 새 테이블 생성
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("error_log"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(Alias::new("error_type"))
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Alias::new("error_message"))
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Alias::new("stack_trace"))
                            .text()
                            .null()
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        ColumnDef::new(Alias::new("browser"))
                            .string()
                            .null()
                    )
                    .col(
                        ColumnDef::new(Alias::new("os"))
                            .string()
                            .null()
                    )
                    .col(
                        ColumnDef::new(Alias::new("ip_address"))
                            .string()
                            .null()
                    )
                    .col(
                        ColumnDef::new(Alias::new("user_agent"))
                            .string()
                            .null()
                    )
                    .col(
                        ColumnDef::new(Alias::new("project_id"))
                            .integer()
                            .not_null()
                            .default(1)
                    )
                    .col(
                        ColumnDef::new(Alias::new("issue_id"))
                            .integer()
                            .null()
                    )
                    .col(
                        ColumnDef::new(Alias::new("reported_by"))
                            .integer()
                            .null()
                    )
                    .to_owned()
            )
            .await?;

        // 외래 키 추가
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_error_project")
                    .from(Alias::new("error_log"), Alias::new("project_id"))
                    .to(Alias::new("projects"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned()
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_error_issue")
                    .from(Alias::new("error_log"), Alias::new("issue_id"))
                    .to(Alias::new("issues"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned()
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_error_user")
                    .from(Alias::new("error_log"), Alias::new("reported_by"))
                    .to(Alias::new("users"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 외래 키 삭제
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_error_user")
                    .table(Alias::new("error_log"))
                    .to_owned()
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_error_issue")
                    .table(Alias::new("error_log"))
                    .to_owned()
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_error_project")
                    .table(Alias::new("error_log"))
                    .to_owned()
            )
            .await?;

        // 테이블 삭제
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("error_log"))
                    .to_owned()
            )
            .await
    }
}