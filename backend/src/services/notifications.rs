use sqlx::PgPool;
use uuid::Uuid;

/// Create a notification for a user
pub async fn create_notification(
    pool: &PgPool,
    user_id: Uuid,
    notification_type: &str,
    payload: serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO notifications (id, user_id, type, payload, read, created_at)
        VALUES ($1, $2, $3, $4, false, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(notification_type)
    .bind(payload)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get unread notifications for a user
pub async fn get_unread(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<crate::models::Notification>, sqlx::Error> {
    sqlx::query_as::<_, crate::models::Notification>(
        r#"
        SELECT * FROM notifications
        WHERE user_id = $1 AND read = false
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Mark notifications as read
pub async fn mark_read(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE notifications SET read = true WHERE user_id = $1 AND read = false")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
