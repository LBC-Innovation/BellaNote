use crate::error::{AppError, AppResult};
use crate::llm::ChatTurn;
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
pub struct Artifact {
    pub id: String,
    pub meeting_group_id: String,
    pub title: String,
    pub source_type: String,
    pub status: String,
    pub has_audio: bool,
    pub original_filename: String,
    pub error_message: String,
    pub transcript: String,
    pub segments_json: String,
    pub duration_ms: i64,
    pub created_at: String,
    #[serde(default)]
    pub whisper_model: String,
    #[serde(default)]
    pub original_path: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactComment {
    pub id: String,
    pub artifact_id: String,
    pub time_ms: i64,
    pub body: String,
    pub created_at: String,
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

fn trim_comment_body(body: &str) -> AppResult<String> {
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(AppError::Message("A comment is required.".into()));
    }
    if body.chars().count() > 500 {
        return Err(AppError::Message(
            "Keep comments to 500 characters or fewer.".into(),
        ));
    }
    Ok(body)
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
                original_path TEXT NOT NULL DEFAULT '',
                error_message TEXT NOT NULL DEFAULT '',
                transcript TEXT NOT NULL DEFAULT '',
                segments_json TEXT NOT NULL DEFAULT '[]',
                duration_ms INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS artifact_comments (
                id TEXT PRIMARY KEY,
                artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
                time_ms INTEGER NOT NULL,
                body TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS artifact_comments_artifact
                ON artifact_comments (artifact_id, time_ms);

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
        migrate_artifacts(&conn)?;
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

    pub fn insert_artifact(&self, artifact: &Artifact) -> AppResult<()> {
        let conn = self.lock()?;
        ensure_exists(
            &conn,
            "SELECT 1 FROM meeting_groups WHERE id = ?1",
            &artifact.meeting_group_id,
            "Meeting group",
        )?;
        conn.execute(
            "INSERT INTO artifacts (
                id, meeting_group_id, title, source_type, status, has_audio,
                original_filename, error_message, transcript, segments_json, duration_ms, created_at,
                whisper_model, original_path
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                artifact.id,
                artifact.meeting_group_id,
                artifact.title,
                artifact.source_type,
                artifact.status,
                artifact.has_audio as i64,
                artifact.original_filename,
                artifact.error_message,
                artifact.transcript,
                artifact.segments_json,
                artifact.duration_ms,
                artifact.created_at,
                artifact.whisper_model,
                artifact.original_path
            ],
        )?;
        Ok(())
    }

    pub fn list_artifacts(&self, meeting_group_id: &str) -> AppResult<Vec<Artifact>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, meeting_group_id, title, source_type, status, has_audio,
                    original_filename, error_message, transcript, segments_json, duration_ms, created_at,
                    whisper_model, original_path
             FROM artifacts
             WHERE meeting_group_id = ?1
             ORDER BY created_at ASC",
        )?;
        let rows = stmt
            .query_map(params![meeting_group_id], map_artifact)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_artifact(&self, id: &str) -> AppResult<Artifact> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT id, meeting_group_id, title, source_type, status, has_audio,
                    original_filename, error_message, transcript, segments_json, duration_ms, created_at,
                    whisper_model, original_path
             FROM artifacts WHERE id = ?1",
            params![id],
            map_artifact,
        )
        .map_err(|_| AppError::Message("Artifact not found.".into()))
    }

    pub fn rename_artifact(&self, id: &str, title: &str) -> AppResult<Artifact> {
        let title = trim_name(title)?;
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE artifacts SET title = ?1 WHERE id = ?2",
            params![title, id],
        )?;
        if changed == 0 {
            return Err(AppError::Message("Artifact not found.".into()));
        }
        drop(conn);
        self.get_artifact(id)
    }

    pub fn delete_artifact(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        let changed = conn.execute("DELETE FROM artifacts WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(AppError::Message("Artifact not found.".into()));
        }
        Ok(())
    }

    pub fn list_artifact_comments(&self, artifact_id: &str) -> AppResult<Vec<ArtifactComment>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, artifact_id, time_ms, body, created_at
             FROM artifact_comments
             WHERE artifact_id = ?1
             ORDER BY time_ms ASC, created_at ASC",
        )?;
        let rows = stmt.query_map(params![artifact_id], map_comment)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn create_artifact_comment(
        &self,
        artifact_id: &str,
        time_ms: i64,
        body: &str,
    ) -> AppResult<ArtifactComment> {
        let body = trim_comment_body(body)?;
        let artifact = self.get_artifact(artifact_id)?;
        if !artifact.has_audio {
            return Err(AppError::Message(
                "Comments can only be added on audio files.".into(),
            ));
        }
        let time_ms = time_ms.max(0);
        let time_ms = if artifact.duration_ms > 0 {
            time_ms.min(artifact.duration_ms)
        } else {
            time_ms
        };
        let comment = ArtifactComment {
            id: new_id(),
            artifact_id: artifact_id.to_string(),
            time_ms,
            body,
            created_at: now_rfc3339(),
        };
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO artifact_comments (id, artifact_id, time_ms, body, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                comment.id,
                comment.artifact_id,
                comment.time_ms,
                comment.body,
                comment.created_at
            ],
        )?;
        Ok(comment)
    }

    pub fn update_artifact_comment(&self, id: &str, body: &str) -> AppResult<ArtifactComment> {
        let body = trim_comment_body(body)?;
        let conn = self.lock()?;
        let changed = conn.execute(
            "UPDATE artifact_comments SET body = ?1 WHERE id = ?2",
            params![body, id],
        )?;
        if changed == 0 {
            return Err(AppError::Message("Comment not found.".into()));
        }
        drop(conn);
        self.get_artifact_comment(id)
    }

    pub fn delete_artifact_comment(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        let changed = conn.execute("DELETE FROM artifact_comments WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(AppError::Message("Comment not found.".into()));
        }
        Ok(())
    }

    pub fn get_artifact_comment(&self, id: &str) -> AppResult<ArtifactComment> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT id, artifact_id, time_ms, body, created_at
             FROM artifact_comments WHERE id = ?1",
            params![id],
            map_comment,
        )
        .map_err(|_| AppError::Message("Comment not found.".into()))
    }

    pub fn fail_interrupted_imports(&self) -> AppResult<usize> {
        let conn = self.lock()?;
        let import_failed = conn.execute(
            "UPDATE artifacts
             SET status = 'failed',
                 error_message = 'Import didn’t finish because BellaNote closed. You can try again.'
             WHERE status IN ('queued', 'transcribing')
               AND source_type NOT IN ('voice', 'system')",
            [],
        )?;
        let record_failed = conn.execute(
            "UPDATE artifacts
             SET status = 'failed',
                 error_message = 'Recording didn’t finish because BellaNote closed.'
             WHERE status IN ('recording', 'queued', 'transcribing')
               AND source_type IN ('voice', 'system')",
            [],
        )?;
        Ok(import_failed + record_failed)
    }

    pub fn set_artifact_status(
        &self,
        id: &str,
        status: &str,
        error_message: &str,
    ) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE artifacts SET status = ?1, error_message = ?2 WHERE id = ?3",
            params![status, error_message, id],
        )?;
        Ok(())
    }

    pub fn set_artifact_live_transcript(
        &self,
        id: &str,
        transcript: &str,
        segments_json: &str,
        duration_ms: i64,
    ) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE artifacts
             SET transcript = ?1, segments_json = ?2, duration_ms = ?3
             WHERE id = ?4",
            params![transcript, segments_json, duration_ms, id],
        )?;
        Ok(())
    }

    pub fn set_artifact_transcript(
        &self,
        id: &str,
        transcript: &str,
        segments_json: &str,
        duration_ms: i64,
        whisper_model: &str,
    ) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE artifacts
             SET transcript = ?1, segments_json = ?2, duration_ms = ?3, status = 'ready',
                 error_message = '', whisper_model = ?4
             WHERE id = ?5",
            params![transcript, segments_json, duration_ms, whisper_model, id],
        )?;
        Ok(())
    }

    pub fn ready_artifacts_for_scope(
        &self,
        scope_type: &str,
        scope_id: &str,
    ) -> AppResult<Vec<Artifact>> {
        let sql = match scope_type {
            "artifact" => {
                "SELECT id, meeting_group_id, title, source_type, status, has_audio,
                        original_filename, error_message, transcript, segments_json, duration_ms, created_at,
                        whisper_model, original_path
                 FROM artifacts WHERE id = ?1 AND status = 'ready'"
            }
            "meeting_group" => {
                "SELECT id, meeting_group_id, title, source_type, status, has_audio,
                        original_filename, error_message, transcript, segments_json, duration_ms, created_at,
                        whisper_model, original_path
                 FROM artifacts WHERE meeting_group_id = ?1 AND status = 'ready'
                 ORDER BY created_at DESC"
            }
            "topic" => {
                "SELECT a.id, a.meeting_group_id, a.title, a.source_type, a.status, a.has_audio,
                        a.original_filename, a.error_message, a.transcript, a.segments_json, a.duration_ms, a.created_at,
                        a.whisper_model, a.original_path
                 FROM artifacts a
                 JOIN meeting_groups g ON g.id = a.meeting_group_id
                 WHERE g.topic_id = ?1 AND a.status = 'ready'
                 ORDER BY g.occurred_at DESC, a.created_at DESC"
            }
            "organization" => {
                "SELECT a.id, a.meeting_group_id, a.title, a.source_type, a.status, a.has_audio,
                        a.original_filename, a.error_message, a.transcript, a.segments_json, a.duration_ms, a.created_at,
                        a.whisper_model, a.original_path
                 FROM artifacts a
                 JOIN meeting_groups g ON g.id = a.meeting_group_id
                 JOIN topic_categories t ON t.id = g.topic_id
                 WHERE t.organization_id = ?1 AND a.status = 'ready'
                 ORDER BY g.occurred_at DESC, a.created_at DESC"
            }
            _ => return Err(AppError::Message("Unknown chat scope.".into())),
        };
        let conn = self.lock()?;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt
            .query_map(params![scope_id], map_artifact)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn count_artifacts_in_scope(
        &self,
        scope_type: &str,
        scope_id: &str,
    ) -> AppResult<(i64, i64)> {
        let ready = self.ready_artifacts_for_scope(scope_type, scope_id)?.len() as i64;
        let total_sql = match scope_type {
            "artifact" => "SELECT COUNT(*) FROM artifacts WHERE id = ?1",
            "meeting_group" => "SELECT COUNT(*) FROM artifacts WHERE meeting_group_id = ?1",
            "topic" => {
                "SELECT COUNT(*) FROM artifacts a
                 JOIN meeting_groups g ON g.id = a.meeting_group_id
                 WHERE g.topic_id = ?1"
            }
            "organization" => {
                "SELECT COUNT(*) FROM artifacts a
                 JOIN meeting_groups g ON g.id = a.meeting_group_id
                 JOIN topic_categories t ON t.id = g.topic_id
                 WHERE t.organization_id = ?1"
            }
            _ => return Err(AppError::Message("Unknown chat scope.".into())),
        };
        let conn = self.lock()?;
        let total: i64 = conn.query_row(total_sql, params![scope_id], |row| row.get(0))?;
        Ok((ready, total))
    }

    pub fn get_chat_messages(&self, scope_type: &str, scope_id: &str) -> AppResult<Vec<ChatTurn>> {
        let conn = self.lock()?;
        let raw: Option<String> = conn
            .query_row(
                "SELECT messages_json FROM chat_threads WHERE scope_type = ?1 AND scope_id = ?2",
                params![scope_type, scope_id],
                |row| row.get(0),
            )
            .optional()?;
        match raw {
            Some(json) => Ok(serde_json::from_str(&json).unwrap_or_default()),
            None => Ok(vec![]),
        }
    }

    pub fn set_chat_messages(
        &self,
        scope_type: &str,
        scope_id: &str,
        messages: &[ChatTurn],
    ) -> AppResult<()> {
        let json = serde_json::to_string(messages)
            .map_err(|e| AppError::Message(e.to_string()))?;
        let now = now_rfc3339();
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO chat_threads (id, scope_type, scope_id, messages_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(scope_type, scope_id) DO UPDATE SET
                messages_json = excluded.messages_json,
                updated_at = excluded.updated_at",
            params![new_id(), scope_type, scope_id, json, now],
        )?;
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

fn map_comment(row: &rusqlite::Row<'_>) -> rusqlite::Result<ArtifactComment> {
    Ok(ArtifactComment {
        id: row.get(0)?,
        artifact_id: row.get(1)?,
        time_ms: row.get(2)?,
        body: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn map_artifact(row: &rusqlite::Row<'_>) -> rusqlite::Result<Artifact> {
    Ok(Artifact {
        id: row.get(0)?,
        meeting_group_id: row.get(1)?,
        title: row.get(2)?,
        source_type: row.get(3)?,
        status: row.get(4)?,
        has_audio: row.get::<_, i64>(5)? != 0,
        original_filename: row.get(6)?,
        error_message: row.get(7)?,
        transcript: row.get(8)?,
        segments_json: row.get(9)?,
        duration_ms: row.get(10)?,
        created_at: row.get(11)?,
        whisper_model: row.get(12)?,
        original_path: row.get(13)?,
    })
}

fn migrate_artifacts(conn: &Connection) -> AppResult<()> {
    let _ = conn.execute(
        "ALTER TABLE artifacts ADD COLUMN whisper_model TEXT NOT NULL DEFAULT ''",
        [],
    );
    conn.execute(
        "UPDATE artifacts
         SET whisper_model = 'small.en'
         WHERE whisper_model = '' AND source_type = 'audio_upload' AND status = 'ready'",
        [],
    )?;
    conn.execute(
        "UPDATE artifacts
         SET whisper_model = 'imported'
         WHERE whisper_model = '' AND source_type = 'transcript_import'",
        [],
    )?;
    let _ = conn.execute(
        "ALTER TABLE artifacts ADD COLUMN original_path TEXT NOT NULL DEFAULT ''",
        [],
    );
    Ok(())
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
