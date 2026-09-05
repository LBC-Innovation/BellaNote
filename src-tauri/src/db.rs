use crate::error::{AppError, AppResult};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

pub struct Db {
    conn: Mutex<Connection>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MeetingGroup {
    pub id: String,
    pub topic_id: String,
    pub name: String,
    pub occurred_at: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTopic {
    #[serde(flatten)]
    pub topic: Topic,
    pub groups: Vec<MeetingGroup>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LibraryOrganization {
    #[serde(flatten)]
    pub organization: Organization,
    pub topics: Vec<LibraryTopic>,
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

fn trim_name(name: &str) -> AppResult<String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Message("A name is required.".into()));
    }
    if name.chars().count() > 80 {
        return Err(AppError::Message(
            "Keep names to 80 characters or fewer.".into(),
        ));
    }
    Ok(name)
}

impl Db {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::Message(format!("create app data: {e}")))?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS organizations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS organizations_name_ci
                ON organizations (lower(name));

            CREATE TABLE IF NOT EXISTS topic_categories (
                id TEXT PRIMARY KEY,
                organization_id TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS topics_name_ci
                ON topic_categories (organization_id, lower(name));

            CREATE TABLE IF NOT EXISTS meeting_groups (
                id TEXT PRIMARY KEY,
                topic_id TEXT NOT NULL REFERENCES topic_categories(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                occurred_at TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS artifacts (
                id TEXT PRIMARY KEY,
                meeting_group_id TEXT NOT NULL REFERENCES meeting_groups(id) ON DELETE CASCADE,
                title TEXT NOT NULL,
                source_type TEXT NOT NULL,
                status TEXT NOT NULL,
                has_audio INTEGER NOT NULL DEFAULT 0,
                original_filename TEXT NOT NULL DEFAULT '',
                error_message TEXT NOT NULL DEFAULT '',
                transcript TEXT NOT NULL DEFAULT '',
                segments_json TEXT NOT NULL DEFAULT '[]',
                duration_ms INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS chat_threads (
                id TEXT PRIMARY KEY,
                scope_type TEXT NOT NULL,
                scope_id TEXT NOT NULL,
                messages_json TEXT NOT NULL DEFAULT '[]',
                updated_at TEXT NOT NULL,
                UNIQUE (scope_type, scope_id)
            );
            ",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> AppResult<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| AppError::Message("database lock poisoned".into()))
    }

    pub fn get_library(&self) -> AppResult<Vec<LibraryOrganization>> {
        let conn = self.lock()?;
        let mut org_stmt = conn.prepare(
            "SELECT id, name, created_at FROM organizations ORDER BY lower(name) ASC",
        )?;
        let orgs = org_stmt
            .query_map([], |row| {
                Ok(Organization {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut out = Vec::with_capacity(orgs.len());
        for org in orgs {
            let mut topic_stmt = conn.prepare(
                "SELECT id, organization_id, name, created_at
                 FROM topic_categories
                 WHERE organization_id = ?1
                 ORDER BY lower(name) ASC",
            )?;
            let topics = topic_stmt
                .query_map(params![org.id], |row| {
                    Ok(Topic {
                        id: row.get(0)?,
                        organization_id: row.get(1)?,
                        name: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            let mut library_topics = Vec::with_capacity(topics.len());
            for topic in topics {
                let mut group_stmt = conn.prepare(
                    "SELECT id, topic_id, name, occurred_at, created_at
                     FROM meeting_groups
                     WHERE topic_id = ?1
                     ORDER BY occurred_at DESC, created_at DESC",
                )?;
                let groups = group_stmt
                    .query_map(params![topic.id], |row| {
                        Ok(MeetingGroup {
                            id: row.get(0)?,
                            topic_id: row.get(1)?,
                            name: row.get(2)?,
                            occurred_at: row.get(3)?,
                            created_at: row.get(4)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                library_topics.push(LibraryTopic { topic, groups });
            }
            out.push(LibraryOrganization {
                organization: org,
                topics: library_topics,
            });
        }
        Ok(out)
    }

    pub fn create_organization(&self, name: &str) -> AppResult<Organization> {
        let name = trim_name(name)?;
        let conn = self.lock()?;
        if name_taken(&conn, "SELECT 1 FROM organizations WHERE lower(name) = lower(?1)", &name)? {
            return Err(AppError::Message(
                "An organization with that name already exists.".into(),
            ));
        }
        let org = Organization {
            id: new_id(),
            name,
            created_at: now_rfc3339(),
        };
        conn.execute(
            "INSERT INTO organizations (id, name, created_at) VALUES (?1, ?2, ?3)",
            params![org.id, org.name, org.created_at],
        )?;
        Ok(org)
    }

    pub fn rename_organization(&self, id: &str, name: &str) -> AppResult<Organization> {
        let name = trim_name(name)?;
        let conn = self.lock()?;
        if name_taken(
            &conn,
            "SELECT 1 FROM organizations WHERE lower(name) = lower(?1) AND id != ?2",
            &name,
        )
        .is_ok()
            && conn
                .query_row(
                    "SELECT 1 FROM organizations WHERE lower(name) = lower(?1) AND id != ?2",
                    params![name, id],
                    |_| Ok(()),
                )
                .optional()?
                .is_some()
        {
            return Err(AppError::Message(
                "An organization with that name already exists.".into(),
            ));
        }
        let changed = conn.execute(
            "UPDATE organizations SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        if changed == 0 {
            return Err(AppError::Message("Organization not found.".into()));
        }
        get_organization(&conn, id)
    }

    pub fn delete_organization(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        let changed = conn.execute("DELETE FROM organizations WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(AppError::Message("Organization not found.".into()));
        }
        Ok(())
    }

    pub fn create_topic(&self, organization_id: &str, name: &str) -> AppResult<Topic> {
        let name = trim_name(name)?;
        let conn = self.lock()?;
        ensure_exists(&conn, "SELECT 1 FROM organizations WHERE id = ?1", organization_id, "Organization")?;
        if conn
            .query_row(
                "SELECT 1 FROM topic_categories WHERE organization_id = ?1 AND lower(name) = lower(?2)",
                params![organization_id, name],
                |_| Ok(()),
            )
            .optional()?
            .is_some()
        {
            return Err(AppError::Message(
                "That topic already exists in this organization.".into(),
            ));
        }
        let topic = Topic {
            id: new_id(),
            organization_id: organization_id.to_string(),
            name,
            created_at: now_rfc3339(),
        };
        conn.execute(
            "INSERT INTO topic_categories (id, organization_id, name, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![topic.id, topic.organization_id, topic.name, topic.created_at],
        )?;
        Ok(topic)
    }

    pub fn rename_topic(&self, id: &str, name: &str) -> AppResult<Topic> {
        let name = trim_name(name)?;
        let conn = self.lock()?;
        let org_id: String = conn
            .query_row(
                "SELECT organization_id FROM topic_categories WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| AppError::Message("Topic not found.".into()))?;
        if conn
            .query_row(
                "SELECT 1 FROM topic_categories WHERE organization_id = ?1 AND lower(name) = lower(?2) AND id != ?3",
                params![org_id, name, id],
                |_| Ok(()),
            )
            .optional()?
            .is_some()
        {
            return Err(AppError::Message(
                "That topic already exists in this organization.".into(),
            ));
        }
        conn.execute(
            "UPDATE topic_categories SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        get_topic(&conn, id)
    }

    pub fn delete_topic(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        let changed = conn.execute("DELETE FROM topic_categories WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(AppError::Message("Topic not found.".into()));
        }
        Ok(())
    }

    pub fn create_meeting_group(
        &self,
        topic_id: &str,
        name: &str,
        occurred_at: Option<String>,
    ) -> AppResult<MeetingGroup> {
        let name = trim_name(name)?;
        let occurred_at = occurred_at
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(now_rfc3339);
        let conn = self.lock()?;
        ensure_exists(
            &conn,
            "SELECT 1 FROM topic_categories WHERE id = ?1",
            topic_id,
            "Topic",
        )?;
        let group = MeetingGroup {
            id: new_id(),
            topic_id: topic_id.to_string(),
            name,
            occurred_at,
            created_at: now_rfc3339(),
        };
        conn.execute(
            "INSERT INTO meeting_groups (id, topic_id, name, occurred_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                group.id,
                group.topic_id,
                group.name,
                group.occurred_at,
                group.created_at
            ],
        )?;
        Ok(group)
    }

    pub fn rename_meeting_group(&self, id: &str, name: &str) -> AppResult<MeetingGroup> {
        let name = trim_name(name)?;
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE meeting_groups SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        if changed == 0 {
            return Err(AppError::Message("Meeting group not found.".into()));
        }
        get_meeting_group(&conn, id)
    }

    pub fn delete_meeting_group(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        let changed = conn.execute("DELETE FROM meeting_groups WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(AppError::Message("Meeting group not found.".into()));
        }
        Ok(())
    }
}

fn name_taken(conn: &Connection, sql: &str, name: &str) -> AppResult<bool> {
    Ok(conn
        .query_row(sql, params![name], |_| Ok(()))
        .optional()?
        .is_some())
}

fn ensure_exists(conn: &Connection, sql: &str, id: &str, label: &str) -> AppResult<()> {
    conn.query_row(sql, params![id], |_| Ok(()))
        .optional()?
        .ok_or_else(|| AppError::Message(format!("{label} not found.")))
}

fn get_organization(conn: &Connection, id: &str) -> AppResult<Organization> {
    conn.query_row(
        "SELECT id, name, created_at FROM organizations WHERE id = ?1",
        params![id],
        |row| {
            Ok(Organization {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        },
    )
    .map_err(|_| AppError::Message("Organization not found.".into()))
}

fn get_topic(conn: &Connection, id: &str) -> AppResult<Topic> {
    conn.query_row(
        "SELECT id, organization_id, name, created_at FROM topic_categories WHERE id = ?1",
        params![id],
        |row| {
            Ok(Topic {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
            })
        },
    )
    .map_err(|_| AppError::Message("Topic not found.".into()))
}

fn get_meeting_group(conn: &Connection, id: &str) -> AppResult<MeetingGroup> {
    conn.query_row(
        "SELECT id, topic_id, name, occurred_at, created_at FROM meeting_groups WHERE id = ?1",
        params![id],
        |row| {
            Ok(MeetingGroup {
                id: row.get(0)?,
                topic_id: row.get(1)?,
                name: row.get(2)?,
                occurred_at: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )
    .map_err(|_| AppError::Message("Meeting group not found.".into()))
}
