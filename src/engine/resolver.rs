use sqlx::SqlitePool;

use crate::api::live::{Broadcaster, LiveEvent};
use crate::models::{agent, matchup, vote};

pub async fn run_resolver(pool: &SqlitePool, broadcaster: &Broadcaster) {
    if let Err(e) = resolve_expired(pool, broadcaster).await {
        tracing::error!("Resolver error: {e}");
    }
}

async fn resolve_expired(pool: &SqlitePool, broadcaster: &Broadcaster) -> Result<(), sqlx::Error> {
    let expired = matchup::get_expired_matchups(pool).await?;

    for m in expired {
        let tally = vote::get_tally(pool, &m.id).await?;
        let total = tally.votes_a + tally.votes_b;

        if total < 5 {
            matchup::resolve_matchup(pool, &m.id, None, "discarded").await?;
            tracing::info!("Discarded matchup {} (only {total} votes)", m.id);
            let _ = broadcaster.send(LiveEvent::MatchupResolved {
                matchup_id: m.id.clone(),
                winner: None,
                hot_take: None,
            });
            continue;
        }

        if tally.votes_a == tally.votes_b {
            matchup::resolve_matchup(pool, &m.id, None, "resolved").await?;
            tracing::info!(
                "Matchup {} tied ({} - {})",
                m.id,
                tally.votes_a,
                tally.votes_b
            );
            let _ = broadcaster.send(LiveEvent::MatchupResolved {
                matchup_id: m.id.clone(),
                winner: None,
                hot_take: None,
            });
            continue;
        }

        let (winner_id, loser_id) = if tally.votes_a > tally.votes_b {
            (&m.agent_a_id, &m.agent_b_id)
        } else {
            (&m.agent_b_id, &m.agent_a_id)
        };

        update_elo(pool, winner_id, loser_id).await?;
        matchup::resolve_matchup(pool, &m.id, Some(winner_id), "resolved").await?;

        // Pick a featured "hot take" comment
        let comments = vote::get_comments_for_matchup(pool, &m.id).await?;
        let hot_take = comments.first().and_then(|c| c.comment.clone());

        tracing::info!(
            "Resolved matchup {}: winner={winner_id} ({} - {})",
            m.id,
            tally.votes_a,
            tally.votes_b
        );

        let winner_agent = agent::find_by_id(pool, winner_id).await?;
        let _ = broadcaster.send(LiveEvent::MatchupResolved {
            matchup_id: m.id.clone(),
            winner: winner_agent.map(|a| a.name),
            hot_take,
        });
    }

    Ok(())
}

async fn update_elo(
    pool: &SqlitePool,
    winner_id: &str,
    loser_id: &str,
) -> Result<(), sqlx::Error> {
    let winner = agent::find_by_id(pool, winner_id).await?.unwrap();
    let loser = agent::find_by_id(pool, loser_id).await?.unwrap();

    let k: f64 = 32.0;
    let expected_winner =
        1.0 / (1.0 + 10f64.powf((loser.elo as f64 - winner.elo as f64) / 400.0));
    let expected_loser = 1.0 - expected_winner;

    let new_winner_elo = winner.elo + (k * (1.0 - expected_winner)) as i64;
    let new_loser_elo = loser.elo + (k * (0.0 - expected_loser)) as i64;

    sqlx::query("UPDATE agents SET elo = ?, wins = wins + 1 WHERE id = ?")
        .bind(new_winner_elo)
        .bind(winner_id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE agents SET elo = ?, losses = losses + 1 WHERE id = ?")
        .bind(new_loser_elo)
        .bind(loser_id)
        .execute(pool)
        .await?;

    Ok(())
}
