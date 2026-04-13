use super::*;

const BUILTIN_CHANNEL_SEEDS: [(&str, &str, i64); 5] = [
    ("openai", "OpenAI", 10),
    ("anthropic", "Anthropic", 20),
    ("gemini", "Gemini", 30),
    ("openrouter", "OpenRouter", 40),
    ("ollama", "Ollama", 50),
];

pub(crate) async fn seed_sqlite_builtin_channels(pool: &SqlitePool) -> Result<()> {
    for (channel_id, channel_name, sort_order) in BUILTIN_CHANNEL_SEEDS {
        sqlx::query(
            "INSERT INTO ai_channel (
                channel_id,
                channel_name,
                channel_description,
                sort_order,
                is_builtin,
                is_active,
                created_at_ms,
                updated_at_ms
            ) VALUES (?, ?, '', ?, 1, 1, 0, 0)
            ON CONFLICT(channel_id) DO UPDATE SET
                channel_name = excluded.channel_name,
                sort_order = excluded.sort_order,
                is_builtin = 1,
                is_active = 1",
        )
        .bind(channel_id)
        .bind(channel_name)
        .bind(sort_order)
        .execute(pool)
        .await?;
    }

    Ok(())
}
