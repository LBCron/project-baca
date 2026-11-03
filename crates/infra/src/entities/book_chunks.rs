//! SeaORM Entity: book_chunks

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "book_chunks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub book_id: Uuid,
    pub chapter_id: Uuid,
    pub chunk_index: i32,
    #[sea_orm(column_type = "Text")]
    pub chunk_text: String,
    pub cfi_range: Option<String>,
    pub embedding: PgVector,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::books::Entity",
        from = "Column::BookId",
        to = "super::books::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Books,
    #[sea_orm(
        belongs_to = "super::chapters::Entity",
        from = "Column::ChapterId",
        to = "super::chapters::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Chapters,
}

impl Related<super::books::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Books.def()
    }
}

impl Related<super::chapters::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Chapters.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
