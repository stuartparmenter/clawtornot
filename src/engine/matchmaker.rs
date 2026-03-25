use sqlx::SqlitePool;

use crate::api::live::{Broadcaster, LiveEvent};
use crate::models::{agent, matchup};

pub async fn run_matchmaker(pool: &SqlitePool, broadcaster: &Broadcaster) {
    if let Err(e) = generate_matchups(pool, broadcaster).await {
        tracing::error!("Matchmaker error: {e}");
    }
}

async fn generate_matchups(pool: &SqlitePool, broadcaster: &Broadcaster) -> Result<(), sqlx::Error> {
    let agent_count = agent::count_agents(pool).await?;
    if agent_count < 3 {
        tracing::info!(
            "Not enough agents ({agent_count}) to generate matchups, need at least 3"
        );
        return Ok(());
    }

    let active = matchup::count_active_matchups(pool).await?;
    let target = (agent_count / 3).max(1).min(20);

    let to_create = target - active;
    if to_create < 1 {
        tracing::info!("Already at target ({active} active, target {target})");
        return Ok(());
    }

    let agents = agent::get_gallery(pool, 1000, 0).await?;

    // Build candidate pairs up front (no rng held across await)
    let candidate_pairs: Vec<(usize, usize)> = {
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();

        let now = chrono::Utc::now().naive_utc();
        let cutoff = now - chrono::Duration::hours(48);
        let mut indices: Vec<usize> = Vec::new();
        for (i, a) in agents.iter().enumerate() {
            indices.push(i);
            if let Ok(created) =
                chrono::NaiveDateTime::parse_from_str(&a.created_at, "%Y-%m-%d %H:%M:%S")
            {
                if created > cutoff {
                    indices.push(i); // double weight for new agents
                }
            }
        }

        let mut pairs = Vec::new();
        for _ in 0..to_create * 3 {
            indices.shuffle(&mut rng);
            if indices.len() < 2 {
                break;
            }
            let ai = indices[0];
            let bi = match indices.iter().find(|&&x| x != ai) {
                Some(&b) => b,
                None => continue,
            };
            pairs.push((ai, bi));
        }
        pairs
    };

    let mut created = 0i64;
    for (ai, bi) in candidate_pairs {
        if created >= to_create {
            break;
        }
        let a = &agents[ai];
        let b = &agents[bi];

        if matchup::recent_pair_exists(pool, &a.id, &b.id).await? {
            continue;
        }

        match matchup::create_matchup(pool, &a.id, &b.id).await {
            Ok(id) => {
                tracing::info!("Created matchup {id}: {} vs {}", a.name, b.name);
                let _ = broadcaster.send(LiveEvent::MatchupCreated {
                    matchup_id: id,
                    agent_a: a.name.clone(),
                    agent_b: b.name.clone(),
                });
                created += 1;
            }
            Err(e) => {
                tracing::warn!("Failed to create matchup: {e}");
            }
        }
    }

    tracing::info!("Matchmaker created {created} new matchups");
    Ok(())
}
