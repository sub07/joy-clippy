use std::fs;

use directories::ProjectDirs;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tracing::info;

use crate::{APPLICATION, ORGANIZATION, QUALIFIER};

const DB_NAME: &str = "clippy.sqlite";

pub async fn get_db() -> anyhow::Result<DatabaseConnection> {
    let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or(anyhow::anyhow!("Could not get os dirs"))?;
    let mut data_dir = dirs.data_dir().to_path_buf();
    if !data_dir.try_exists()? {
        fs::create_dir_all(&data_dir)?;
    }

    data_dir.push(DB_NAME);

    let db_url = format!("sqlite://{}?mode=rwc", data_dir.display());
    info!("Connecting to sqlite db: {db_url}");
    let db = Database::connect(db_url).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

pub mod repo {
    use chrono::Utc;
    use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder, Set};

    pub async fn add_item(db: &DatabaseConnection, data: String) -> anyhow::Result<()> {
        entity::entry::Entity::insert(entity::entry::ActiveModel {
            data: Set(data),
            added_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        })
        .exec_without_returning(db)
        .await?;
        Ok(())
    }

    pub async fn get_items(db: &DatabaseConnection) -> anyhow::Result<Vec<entity::entry::Model>> {
        Ok(entity::entry::Entity::find()
            .order_by_desc(entity::entry::Column::AddedAt)
            .all(db)
            .await?)
    }

    pub async fn delete(
        db: &DatabaseConnection,
        entry: &entity::entry::Model,
    ) -> anyhow::Result<()> {
        entity::entry::Entity::delete_by_id(entry.id)
            .exec(db)
            .await?;
        Ok(())
    }
}
