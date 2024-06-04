const MIGRATION_TABLE: &str = r#"

CREATE TABLE IF NOT EXISTS
    fastn_migration
(
    id               INTEGER PRIMARY KEY,
    app_name         TEXT NOT NULL,
    migration_number INTEGER NOT NULL UNIQUE,
    migration_name   TEXT NOT NULL,
    applied_on       INTEGER NOT NULL
) STRICT;

"#;

pub(crate) async fn migrate(
    req_config: &mut fastn_core::RequestConfig,
) -> Result<(), MigrationError> {
    let db = req_config.config.get_db_url().await;
    create_migration_table(&req_config.config, db.as_str()).await?;

    let available_migrations = req_config.config.package.migration.clone();
    let applied_migrations = get_applied_migrations(&req_config.config, db.as_str()).await?;

    if !has_new_migration(
        applied_migrations.as_slice(),
        available_migrations.as_slice(),
    )? {
        return Ok(());
    }

    let migrations_to_apply =
        find_migrations_to_apply(&available_migrations, &applied_migrations).await?;

    let now = chrono::Utc::now();
    for migration in migrations_to_apply {
        req_config
            .config
            .ds
            .sql_batch(db.as_str(), migration.content.as_str())
            .await?;
        mark_migration_applied(&req_config.config, db.as_str(), &migration, &now).await?;
    }

    Ok(())
}

async fn create_migration_table(
    config: &fastn_core::Config,
    db: &str,
) -> Result<(), fastn_utils::SqlError> {
    config.ds.sql_batch(db, MIGRATION_TABLE).await?;
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum MigrationError {
    #[error("Cannot delete applied migration")]
    DeleteAppliedMigration,
    #[error("Sql Error: {0}")]
    SqlError(#[from] fastn_utils::SqlError),
    #[error("The migration order has changed or its content has been altered")]
    AppliedMigrationMismatch,
}

fn has_new_migration(
    applied_migrations: &[MigrationDataSQL],
    available_migrations: &[fastn_core::package::MigrationData],
) -> Result<bool, MigrationError> {
    if applied_migrations.len() > available_migrations.len() {
        return Err(MigrationError::DeleteAppliedMigration);
    } else if applied_migrations.len() < available_migrations.len() {
        return Ok(true);
    } else {
        Ok(false)
    }
}

async fn mark_migration_applied(
    config: &fastn_core::Config,
    db: &str,
    migration_data: &fastn_core::package::MigrationData,
    now: &chrono::DateTime<chrono::Utc>,
) -> Result<(), fastn_utils::SqlError> {
    let now_in_nanosecond = now.timestamp_nanos_opt().unwrap();
    config
        .ds
        .sql_execute(
            db,
            format!(
                r#"
                    INSERT INTO
                        fastn_migration
                            (app_name, migration_number, migration_name, applied_on)
                    VALUES
                        ({}, {}, {}, {});
                "#,
                config.package.name, migration_data.number, migration_data.name, now_in_nanosecond
            )
            .as_str(),
            vec![],
        )
        .await?;
    Ok(())
}

#[derive(Clone)]
struct MigrationDataSQL {
    number: i64,
    name: String,
}

async fn find_migrations_to_apply(
    available_migrations: &[fastn_core::package::MigrationData],
    applied_migrations: &[MigrationDataSQL],
) -> Result<Vec<fastn_core::package::MigrationData>, MigrationError> {
    let applied_migrations: std::collections::HashMap<i64, MigrationDataSQL> = applied_migrations
        .into_iter()
        .map(|val| (val.number, val.clone()))
        .collect();

    let mut migrations_to_apply = vec![];
    for m_ftd in available_migrations {
        match applied_migrations.get(&m_ftd.number) {
            Some(m_sql) => {
                if m_sql.name.ne(&m_ftd.name) {
                    return Err(MigrationError::AppliedMigrationMismatch);
                }
            }
            None => {
                migrations_to_apply.push(m_ftd.clone());
            }
        }
    }

    Ok(migrations_to_apply)
}

async fn get_applied_migrations(
    config: &fastn_core::Config,
    db: &str,
) -> Result<Vec<MigrationDataSQL>, fastn_utils::SqlError> {
    let results = config
        .ds
        .sql_query(
            db,
            format!(
                r#"
                    SELECT
                        migration_number, migration_name
                    FROM
                        fastn_migration
                    WHERE
                        app_name='{}'
                "#,
                config.package.name
            )
            .as_str(),
            vec![],
        )
        .await?;

    let mut migration_details = vec![];

    for entry in results {
        let migration_number: i64 = serde_json::from_value(
            entry
                .get(0)
                .expect("fastn_migration::migration_number not found.")
                .clone(),
        )
        .unwrap();
        let migration_name: String = serde_json::from_value(
            entry
                .get(1)
                .expect("fastn_migration::migration_name not found.")
                .clone(),
        )
        .unwrap();
        migration_details.push(MigrationDataSQL {
            number: migration_number,
            name: migration_name,
        });
    }
    Ok(migration_details)
}
