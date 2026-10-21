use sqlx::{FromRow, MySqlPool};

use super::dto::{AdminCustomerPage, AdminCustomerQuery, MembershipConfig, MembershipTier};

#[derive(Debug, FromRow)]
struct MembershipSettingsRow {
    spend_step_cents: i64,
    points_per_step: i64,
}

#[derive(Debug, FromRow)]
struct MembershipTierRow {
    level: i64,
    required_points: i64,
    discount_rate: f64,
}

#[derive(Clone)]
pub(crate) struct MembershipRepository {
    pool: MySqlPool,
}

impl MembershipRepository {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn config(&self) -> Result<MembershipConfig, sqlx::Error> {
        let settings = sqlx::query_as::<_, MembershipSettingsRow>(
            "SELECT spend_step_cents, points_per_step FROM membership_settings WHERE id = 1",
        )
        .fetch_one(&self.pool)
        .await?;
        let rows = sqlx::query_as::<_, MembershipTierRow>(
            "SELECT level, required_points, CAST(discount_rate AS DOUBLE) AS discount_rate \
             FROM membership_tiers ORDER BY level",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(MembershipConfig {
            spend_step_cents: settings.spend_step_cents,
            points_per_step: settings.points_per_step,
            tiers: rows
                .into_iter()
                .map(|row| MembershipTier {
                    level: row.level,
                    required_points: row.required_points,
                    discount_rate: row.discount_rate,
                })
                .collect(),
        })
    }

    pub(crate) async fn update_config(&self, config: MembershipConfig) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE membership_settings SET spend_step_cents = ?, points_per_step = ? WHERE id = 1",
        )
        .bind(config.spend_step_cents)
        .bind(config.points_per_step)
        .execute(&mut *tx)
        .await?;
        for tier in config.tiers {
            sqlx::query("UPDATE membership_tiers SET required_points = ?, discount_rate = ? WHERE level = ?")
                .bind(tier.required_points)
                .bind(tier.discount_rate)
                .bind(tier.level)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await
    }

    pub(crate) async fn list_customers(
        &self,
        query: AdminCustomerQuery,
    ) -> Result<AdminCustomerPage, sqlx::Error> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(10, 100);
        let keyword = query.q.filter(|value| !value.trim().is_empty());
        let search_id = keyword.as_ref().and_then(|value| value.parse::<i64>().ok());
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM customers c \
             WHERE (? IS NULL OR c.nickname LIKE CONCAT('%', ?, '%') OR c.id = ?)",
        )
        .bind(keyword.clone())
        .bind(keyword.clone())
        .bind(search_id)
        .fetch_one(&self.pool)
        .await?;

        let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
        let customers = sqlx::query_as::<_, super::dto::AdminCustomer>(
            "SELECT c.id, c.nickname, c.points, COALESCE(t.level, 1) AS member_level, \
             COALESCE(CAST(t.discount_rate AS DOUBLE), 1.0) AS discount_rate, \
             COALESCE(o.order_count, 0) AS order_count, CAST(COALESCE(o.paid_total_cents, 0) AS SIGNED) AS paid_total_cents, \
             DATE_FORMAT(c.created_at, '%Y-%m-%d %H:%i:%s') AS created_at \
             FROM customers c \
             LEFT JOIN membership_tiers t ON t.level = (SELECT MAX(mt.level) FROM membership_tiers mt WHERE mt.required_points <= c.points) \
             LEFT JOIN (SELECT customer_id, COUNT(*) AS order_count, SUM(CASE WHEN payment_status = 'paid' THEN total_cents ELSE 0 END) AS paid_total_cents \
                        FROM restaurant_orders GROUP BY customer_id) o ON o.customer_id = c.id \
             WHERE (? IS NULL OR c.nickname LIKE CONCAT('%', ?, '%') OR c.id = ?) \
             ORDER BY c.created_at DESC, c.id DESC LIMIT ? OFFSET ?",
        )
        .bind(keyword.clone())
        .bind(keyword)
        .bind(search_id)
        .bind(i64::from(page_size))
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(AdminCustomerPage {
            customers,
            total,
            page,
            page_size,
        })
    }
}
