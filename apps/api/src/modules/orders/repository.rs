use std::{
    collections::{BTreeMap, HashMap},
    time::{SystemTime, UNIX_EPOCH},
};

use rand::{RngCore, rngs::OsRng};
use sqlx::{FromRow, MySql, MySqlPool, QueryBuilder, types::Json};

use crate::{common::ApiError, modules::products::ProductSpecificationGroup};

use super::{
    dto::{AdminOrderQuery, OrderInput, OrderItemInput, SpecificationSelection},
    entity::{
        AdminOrderPage, Order, OrderItemRecord, OrderItemSummary, OrderRecord, OrderSummary,
        PaidOrderResult,
    },
};

#[derive(Clone)]
pub(crate) struct OrdersRepository {
    pool: MySqlPool,
}

#[derive(Debug, FromRow)]
struct OrderSettings {
    is_open: bool,
    dine_in_enabled: bool,
    takeaway_enabled: bool,
    delivery_enabled: bool,
    delivery_minimum_amount: f64,
    delivery_fee: f64,
}

#[derive(Debug, FromRow)]
struct OrderProduct {
    id: i64,
    name: String,
    category: String,
    specifications: Json<Vec<ProductSpecificationGroup>>,
    price: f64,
    stock: i64,
    status: String,
    discount_eligible: bool,
}

#[derive(Debug, FromRow)]
struct DeliveryAddress {
    recipient_name: String,
    phone: String,
    full_address: String,
}

struct ItemSnapshot {
    product_id: i64,
    product_name: String,
    category: String,
    unit_price_cents: i64,
    quantity: i64,
    line_total_cents: i64,
    discount_cents: i64,
    discount_eligible: bool,
    selections: Vec<SpecificationSelection>,
}

#[derive(Debug, FromRow)]
struct PaymentOrder {
    id: i64,
    customer_id: i64,
    status: String,
    payment_status: String,
    subtotal_cents: i64,
    discount_cents: i64,
    points_earned: i64,
    is_unexpired: bool,
}

#[derive(Debug, FromRow)]
struct DeletableOrder {
    id: i64,
    payment_status: String,
    stock_released: bool,
}

impl OrdersRepository {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn list_for_customer(
        &self,
        customer_id: i64,
    ) -> Result<Vec<OrderSummary>, ApiError> {
        let records = sqlx::query_as::<_, OrderRecord>(
            "SELECT o.id, o.order_number, o.customer_id, \
             COALESCE(NULLIF(c.nickname, ''), CONCAT('微信用户', LPAD(c.id, 6, '0'))) AS customer_name, \
             o.fulfillment_mode, o.table_number, o.delivery_recipient_name, o.delivery_phone, \
             o.delivery_address, o.status, o.payment_status, o.subtotal_cents, o.discount_cents, o.delivery_fee_cents, \
             o.total_cents, o.points_earned, DATE_FORMAT(o.expires_at, '%Y-%m-%d %H:%i:%s') AS expires_at, \
             DATE_FORMAT(o.created_at, '%Y-%m-%d %H:%i:%s') AS created_at \
             FROM restaurant_orders o JOIN customers c ON c.id = o.customer_id \
             WHERE o.customer_id = ? ORDER BY o.created_at DESC, o.id DESC",
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;

        self.attach_items(records).await
    }

    pub(crate) async fn list_for_admin(
        &self,
        query: AdminOrderQuery,
    ) -> Result<AdminOrderPage, ApiError> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);

        let mut count_query = QueryBuilder::<MySql>::new(
            "SELECT COUNT(*) FROM restaurant_orders o \
             JOIN customers c ON c.id = o.customer_id WHERE 1 = 1",
        );
        push_admin_filters(&mut count_query, &query);
        let total = count_query
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await?;

        let mut order_query = QueryBuilder::<MySql>::new(
            "SELECT o.id, o.order_number, o.customer_id, \
             COALESCE(NULLIF(c.nickname, ''), CONCAT('微信用户', LPAD(c.id, 6, '0'))) AS customer_name, \
             o.fulfillment_mode, o.table_number, o.delivery_recipient_name, o.delivery_phone, \
             o.delivery_address, o.status, o.payment_status, o.subtotal_cents, o.discount_cents, o.delivery_fee_cents, \
             o.total_cents, o.points_earned, DATE_FORMAT(o.expires_at, '%Y-%m-%d %H:%i:%s') AS expires_at, \
             DATE_FORMAT(o.created_at, '%Y-%m-%d %H:%i:%s') AS created_at \
             FROM restaurant_orders o JOIN customers c ON c.id = o.customer_id WHERE 1 = 1",
        );
        push_admin_filters(&mut order_query, &query);
        let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
        order_query
            .push(" ORDER BY o.created_at DESC, o.id DESC LIMIT ")
            .push_bind(i64::from(page_size))
            .push(" OFFSET ")
            .push_bind(offset);
        let records = order_query
            .build_query_as::<OrderRecord>()
            .fetch_all(&self.pool)
            .await?;
        let orders = self.attach_items(records).await?;

        Ok(AdminOrderPage {
            orders,
            total,
            page,
            page_size,
        })
    }

    async fn attach_items(&self, records: Vec<OrderRecord>) -> Result<Vec<OrderSummary>, ApiError> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        let mut query = QueryBuilder::<MySql>::new(
            "SELECT order_id, product_name, quantity, unit_price_cents, line_total_cents, discount_cents, discount_eligible, selected_options \
             FROM restaurant_order_items WHERE order_id IN (",
        );
        {
            let mut separated = query.separated(", ");
            for record in &records {
                separated.push_bind(record.id);
            }
        }
        query.push(" ) ORDER BY order_id, id");
        let item_records = query
            .build_query_as::<OrderItemRecord>()
            .fetch_all(&self.pool)
            .await?;
        let mut items_by_order = HashMap::<i64, Vec<OrderItemSummary>>::new();
        for item in item_records {
            items_by_order
                .entry(item.order_id)
                .or_default()
                .push(OrderItemSummary {
                    product_name: item.product_name,
                    quantity: item.quantity,
                    unit_price_cents: item.unit_price_cents,
                    line_total_cents: item.line_total_cents,
                    discount_cents: item.discount_cents,
                    discount_eligible: item.discount_eligible,
                    selected_options: item.selected_options.0,
                });
        }

        Ok(records
            .into_iter()
            .map(|record| OrderSummary {
                order_number: record.order_number,
                customer_id: record.customer_id,
                customer_name: record.customer_name,
                fulfillment_mode: record.fulfillment_mode,
                table_number: record.table_number,
                delivery_recipient_name: record.delivery_recipient_name,
                delivery_phone: record.delivery_phone,
                delivery_address: record.delivery_address,
                status: record.status,
                payment_status: record.payment_status,
                subtotal_cents: record.subtotal_cents,
                discount_cents: record.discount_cents,
                delivery_fee_cents: record.delivery_fee_cents,
                total_cents: record.total_cents,
                points_earned: record.points_earned,
                expires_at: record.expires_at,
                created_at: record.created_at,
                items: items_by_order.remove(&record.id).unwrap_or_default(),
            })
            .collect())
    }

    pub(crate) async fn create(
        &self,
        customer_id: i64,
        input: OrderInput,
    ) -> Result<Order, ApiError> {
        let mut tx = self.pool.begin().await?;

        // Serialize submissions per customer; this makes request-key retries safe and prevents duplicate stock deductions.
        let customer_points: Option<i64> =
            sqlx::query_scalar("SELECT points FROM customers WHERE id = ? FOR UPDATE")
                .bind(customer_id)
                .fetch_optional(&mut *tx)
                .await?;
        let customer_points =
            customer_points.ok_or_else(|| ApiError::unauthorized("登录已过期，请重新登录"))?;
        if customer_points < 0 {
            return Err(ApiError::conflict("会员积分数据异常，请联系门店"));
        }

        if let Some(existing) = sqlx::query_as::<_, Order>(
            "SELECT order_number, fulfillment_mode, status, payment_status, subtotal_cents, discount_cents, \
             delivery_fee_cents, total_cents, points_earned FROM restaurant_orders \
             WHERE customer_id = ? AND request_key = ?",
        )
        .bind(customer_id)
        .bind(&input.request_key)
        .fetch_optional(&mut *tx)
        .await?
        {
            tx.commit().await?;
            return Ok(existing);
        }

        let member_discount_rate: f64 = sqlx::query_scalar(
            "SELECT CAST(discount_rate AS DOUBLE) FROM membership_tiers \
             WHERE required_points <= ? ORDER BY required_points DESC LIMIT 1",
        )
        .bind(customer_points)
        .fetch_optional(&mut *tx)
        .await?
        .unwrap_or(1.0);

        let settings = sqlx::query_as::<_, OrderSettings>(
            "SELECT is_open, dine_in_enabled, takeaway_enabled, delivery_enabled, \
             delivery_minimum_amount, delivery_fee FROM restaurant_settings WHERE id = 1 FOR UPDATE",
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::service_unavailable("门店配置暂不可用"))?;
        if !settings.is_open {
            return Err(ApiError::conflict("门店暂未接单"));
        }
        match input.fulfillment_mode.as_str() {
            "dine_in" if !settings.dine_in_enabled => {
                return Err(ApiError::conflict("门店暂未开放堂食"));
            }
            "takeaway" if !settings.takeaway_enabled => {
                return Err(ApiError::conflict("门店暂未开放自取"));
            }
            "delivery" if !settings.delivery_enabled => {
                return Err(ApiError::conflict("门店暂未开放外卖"));
            }
            _ => {}
        }

        let delivery_address = if input.fulfillment_mode == "delivery" {
            Some(
                sqlx::query_as::<_, DeliveryAddress>(
                    "SELECT recipient_name, phone, CONCAT(province, city, district, detail) AS full_address \
                     FROM customer_addresses WHERE id = ? AND customer_id = ? FOR UPDATE",
                )
                .bind(input.address_id.expect("delivery address validated"))
                .bind(customer_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| ApiError::bad_request("收货地址不存在，请重新选择"))?,
            )
        } else {
            None
        };

        let mut required_stock = BTreeMap::<i64, i64>::new();
        for item in &input.items {
            let required = required_stock.entry(item.product_id).or_default();
            *required = required
                .checked_add(item.quantity)
                .ok_or_else(|| ApiError::bad_request("菜品数量超出范围"))?;
            if *required > 99 {
                return Err(ApiError::bad_request("单道菜最多点 99 份"));
            }
        }

        // Lock product rows in ascending id order, then check current sale state and inventory.
        let mut products = BTreeMap::new();
        for (product_id, quantity) in &required_stock {
            let product = sqlx::query_as::<_, OrderProduct>(
                "SELECT id, name, category, COALESCE(specifications, JSON_ARRAY()) AS specifications, price, stock, status, discount_eligible \
                 FROM products WHERE id = ? FOR UPDATE",
            )
            .bind(product_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| ApiError::conflict("有菜品已下架，请刷新菜单"))?;
            if product.status != "active" {
                return Err(ApiError::conflict(format!("{} 已暂停售卖", product.name)));
            }
            if product.stock < *quantity {
                return Err(ApiError::conflict(format!(
                    "{} 可售份数不足，请调整数量",
                    product.name
                )));
            }
            products.insert(*product_id, product);
        }

        let mut subtotal_cents = 0_i64;
        let mut discount_cents = 0_i64;
        let mut snapshots = Vec::with_capacity(input.items.len());
        for item in &input.items {
            let product = products
                .get(&item.product_id)
                .expect("all requested products were loaded under lock");
            let base_price = to_cents(product.price)?;
            let (unit_price_cents, selections) =
                resolve_selections(&product.specifications.0, item, base_price)?;
            let discounted_unit_price_cents = if product.discount_eligible {
                apply_discount(unit_price_cents, member_discount_rate)?
            } else {
                unit_price_cents
            };
            let original_line_total_cents = unit_price_cents
                .checked_mul(item.quantity)
                .ok_or_else(|| ApiError::bad_request("订单金额超出范围"))?;
            let line_total_cents = discounted_unit_price_cents
                .checked_mul(item.quantity)
                .ok_or_else(|| ApiError::bad_request("订单金额超出范围"))?;
            let line_discount_cents = original_line_total_cents - line_total_cents;
            subtotal_cents = subtotal_cents
                .checked_add(original_line_total_cents)
                .ok_or_else(|| ApiError::bad_request("订单金额超出范围"))?;
            discount_cents = discount_cents
                .checked_add(line_discount_cents)
                .ok_or_else(|| ApiError::bad_request("订单金额超出范围"))?;
            snapshots.push(ItemSnapshot {
                product_id: product.id,
                product_name: product.name.clone(),
                category: product.category.clone(),
                unit_price_cents,
                quantity: item.quantity,
                line_total_cents,
                discount_cents: line_discount_cents,
                discount_eligible: product.discount_eligible,
                selections,
            });
        }

        if input.fulfillment_mode == "delivery" {
            let minimum_cents = to_cents(settings.delivery_minimum_amount)?;
            if subtotal_cents < minimum_cents {
                return Err(ApiError::conflict(format!(
                    "外卖起送金额为 ¥{:.2}",
                    minimum_cents as f64 / 100.0
                )));
            }
        }
        let delivery_fee_cents = if delivery_address.is_some() {
            to_cents(settings.delivery_fee)?
        } else {
            0
        };
        let merchandise_total_cents = subtotal_cents - discount_cents;
        let total_cents = merchandise_total_cents
            .checked_add(delivery_fee_cents)
            .ok_or_else(|| ApiError::bad_request("订单金额超出范围"))?;
        let order_number = make_order_number();
        let (recipient_name, recipient_phone, address) = delivery_address
            .as_ref()
            .map(|address| {
                (
                    Some(address.recipient_name.as_str()),
                    Some(address.phone.as_str()),
                    Some(address.full_address.as_str()),
                )
            })
            .unwrap_or((None, None, None));

        let result = sqlx::query(
            "INSERT INTO restaurant_orders \
             (order_number, customer_id, request_key, fulfillment_mode, table_number, \
              delivery_recipient_name, delivery_phone, delivery_address, status, payment_status, \
              expires_at, subtotal_cents, discount_cents, delivery_fee_cents, total_cents) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'pending_payment', 'unpaid', \
                     DATE_ADD(CURRENT_TIMESTAMP(3), INTERVAL 15 MINUTE), ?, ?, ?, ?)",
        )
        .bind(&order_number)
        .bind(customer_id)
        .bind(&input.request_key)
        .bind(&input.fulfillment_mode)
        .bind(input.table_number.as_deref())
        .bind(recipient_name)
        .bind(recipient_phone)
        .bind(address)
        .bind(subtotal_cents)
        .bind(discount_cents)
        .bind(delivery_fee_cents)
        .bind(total_cents)
        .execute(&mut *tx)
        .await?;
        let order_id = result.last_insert_id() as i64;

        for snapshot in &snapshots {
            sqlx::query(
                "INSERT INTO restaurant_order_items \
                 (order_id, product_id, product_name, category, unit_price_cents, quantity, line_total_cents, discount_cents, discount_eligible, selected_options) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(order_id)
            .bind(snapshot.product_id)
            .bind(&snapshot.product_name)
            .bind(&snapshot.category)
            .bind(snapshot.unit_price_cents)
            .bind(snapshot.quantity)
            .bind(snapshot.line_total_cents)
            .bind(snapshot.discount_cents)
            .bind(snapshot.discount_eligible)
            .bind(Json(snapshot.selections.clone()))
            .execute(&mut *tx)
            .await?;
        }

        for (product_id, quantity) in required_stock {
            let result = sqlx::query(
                "UPDATE products SET stock = stock - ? WHERE id = ? AND status = 'active' AND stock >= ?",
            )
            .bind(quantity)
            .bind(product_id)
            .bind(quantity)
            .execute(&mut *tx)
            .await?;
            if result.rows_affected() != 1 {
                return Err(ApiError::conflict("菜品库存已变化，请刷新后重试"));
            }
        }

        tx.commit().await?;
        Ok(Order {
            order_number,
            fulfillment_mode: input.fulfillment_mode,
            status: "pending_payment".into(),
            payment_status: "unpaid".into(),
            subtotal_cents,
            discount_cents,
            delivery_fee_cents,
            total_cents,
            points_earned: 0,
        })
    }

    pub(crate) async fn mark_paid(&self, order_number: &str) -> Result<PaidOrderResult, ApiError> {
        let mut tx = self.pool.begin().await?;
        let order = sqlx::query_as::<_, PaymentOrder>(
            "SELECT id, customer_id, status, payment_status, subtotal_cents, discount_cents, points_earned, \
             (expires_at > CURRENT_TIMESTAMP(3)) AS is_unexpired \
             FROM restaurant_orders WHERE order_number = ? FOR UPDATE",
        )
        .bind(order_number)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::not_found("订单不存在"))?;

        if order.payment_status == "paid" {
            tx.commit().await?;
            return Ok(PaidOrderResult {
                order_number: order_number.to_owned(),
                payment_status: order.payment_status,
                status: order.status,
                points_earned: order.points_earned,
            });
        }
        if order.status != "pending_payment" || !order.is_unexpired {
            return Err(ApiError::conflict("订单已取消或超时，不能确认收款"));
        }

        let (spend_step_cents, points_per_step): (i64, i64) = sqlx::query_as(
            "SELECT spend_step_cents, points_per_step FROM membership_settings WHERE id = 1",
        )
        .fetch_one(&mut *tx)
        .await?;
        let paid_merchandise_cents = order
            .subtotal_cents
            .checked_sub(order.discount_cents)
            .filter(|amount| *amount >= 0)
            .ok_or_else(|| ApiError::conflict("订单优惠金额异常"))?;
        let points_earned =
            calculate_earned_points(paid_merchandise_cents, spend_step_cents, points_per_step)?;
        let new_points: i64 =
            sqlx::query_scalar("SELECT points FROM customers WHERE id = ? FOR UPDATE")
                .bind(order.customer_id)
                .fetch_one(&mut *tx)
                .await?;
        if new_points < 0 {
            return Err(ApiError::conflict("会员积分数据异常，请联系门店"));
        }
        let _ = new_points
            .checked_add(points_earned)
            .ok_or_else(|| ApiError::conflict("会员积分超出可记录范围"))?;

        sqlx::query("UPDATE restaurant_orders SET payment_status = 'paid', status = 'paid', points_earned = ? WHERE id = ?")
            .bind(points_earned)
            .bind(order.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE customers SET points = points + ? WHERE id = ?")
            .bind(points_earned)
            .bind(order.customer_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        Ok(PaidOrderResult {
            order_number: order_number.to_owned(),
            payment_status: "paid".into(),
            status: "paid".into(),
            points_earned,
        })
    }

    pub(crate) async fn delete_admin_order(&self, order_number: &str) -> Result<(), ApiError> {
        let mut tx = self.pool.begin().await?;
        let order = sqlx::query_as::<_, DeletableOrder>(
            "SELECT id, payment_status, stock_released FROM restaurant_orders \
             WHERE order_number = ? FOR UPDATE",
        )
        .bind(order_number)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| ApiError::not_found("订单不存在"))?;

        if order.payment_status != "unpaid" {
            return Err(ApiError::conflict(
                "已支付订单涉及财务和积分记录，不能直接删除",
            ));
        }

        if !order.stock_released {
            let items: Vec<(i64, i64)> = sqlx::query_as(
                "SELECT product_id, CAST(SUM(quantity) AS SIGNED) FROM restaurant_order_items \
                 WHERE order_id = ? GROUP BY product_id ORDER BY product_id",
            )
            .bind(order.id)
            .fetch_all(&mut *tx)
            .await?;
            for (product_id, quantity) in items {
                sqlx::query("UPDATE products SET stock = stock + ? WHERE id = ?")
                    .bind(quantity)
                    .bind(product_id)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        sqlx::query("DELETE FROM restaurant_orders WHERE id = ? AND payment_status = 'unpaid'")
            .bind(order.id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

fn apply_discount(price_cents: i64, rate: f64) -> Result<i64, ApiError> {
    if !rate.is_finite() || !(0.0..=1.0).contains(&rate) {
        return Err(ApiError::service_unavailable("会员折扣配置无效"));
    }
    let basis_points = (rate * 10_000.0).round() as i64;
    let discounted = (i128::from(price_cents) * i128::from(basis_points) + 5_000) / 10_000;
    i64::try_from(discounted).map_err(|_| ApiError::bad_request("订单金额超出范围"))
}

fn push_admin_filters(query: &mut QueryBuilder<'_, MySql>, filter: &AdminOrderQuery) {
    if let Some(customer_id) = filter.customer_id {
        query.push(" AND o.customer_id = ").push_bind(customer_id);
    }
    if let Some(status) = &filter.status {
        query.push(" AND o.status = ").push_bind(status.clone());
    }
    if let Some(fulfillment_mode) = &filter.fulfillment_mode {
        query
            .push(" AND o.fulfillment_mode = ")
            .push_bind(fulfillment_mode.clone());
    }
    if let Some(search) = &filter.q {
        let pattern = format!("%{search}%");
        query
            .push(" AND (o.order_number LIKE ")
            .push_bind(pattern.clone())
            .push(" OR c.nickname LIKE ")
            .push_bind(pattern.clone())
            .push(" OR o.delivery_phone LIKE ")
            .push_bind(pattern)
            .push(" OR CAST(o.customer_id AS CHAR) = ")
            .push_bind(search.clone())
            .push(")");
    }
}

pub(crate) async fn expire_pending_orders(pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let expired_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM restaurant_orders \
         WHERE status = 'pending_payment' AND payment_status = 'unpaid' \
           AND stock_released = FALSE AND expires_at <= CURRENT_TIMESTAMP(3) \
         ORDER BY expires_at, id LIMIT 100 FOR UPDATE SKIP LOCKED",
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut released_orders = 0_u64;
    for order_id in expired_ids {
        let items: Vec<(i64, i64)> = sqlx::query_as(
            "SELECT product_id, CAST(SUM(quantity) AS SIGNED) FROM restaurant_order_items \
             WHERE order_id = ? GROUP BY product_id ORDER BY product_id",
        )
        .bind(order_id)
        .fetch_all(&mut *tx)
        .await?;
        for (product_id, quantity) in items {
            sqlx::query("UPDATE products SET stock = stock + ? WHERE id = ?")
                .bind(quantity)
                .bind(product_id)
                .execute(&mut *tx)
                .await?;
        }
        let updated = sqlx::query(
            "UPDATE restaurant_orders SET status = 'cancelled', stock_released = TRUE \
             WHERE id = ? AND status = 'pending_payment' AND payment_status = 'unpaid' \
               AND stock_released = FALSE",
        )
        .bind(order_id)
        .execute(&mut *tx)
        .await?;
        released_orders += updated.rows_affected();
    }
    tx.commit().await?;
    Ok(released_orders)
}

fn resolve_selections(
    groups: &[ProductSpecificationGroup],
    item: &OrderItemInput,
    base_price_cents: i64,
) -> Result<(i64, Vec<SpecificationSelection>), ApiError> {
    let mut requested = HashMap::<String, String>::new();
    for selection in &item.selections {
        if requested
            .insert(selection.group_name.clone(), selection.option_name.clone())
            .is_some()
        {
            return Err(ApiError::bad_request("同一规格不能重复选择"));
        }
    }

    let mut price_cents = base_price_cents;
    let mut normalized = Vec::new();
    for group in groups {
        let option_name = requested.remove(&group.name);
        let Some(option_name) = option_name else {
            if group.required {
                return Err(ApiError::bad_request(format!("请选择{}", group.name)));
            }
            continue;
        };
        let option = group
            .options
            .iter()
            .find(|option| option.name == option_name)
            .ok_or_else(|| ApiError::bad_request("菜品规格已更新，请重新选择"))?;
        price_cents = price_cents
            .checked_add(to_cents(option.price_delta)?)
            .ok_or_else(|| ApiError::bad_request("订单金额超出范围"))?;
        normalized.push(SpecificationSelection {
            group_name: group.name.clone(),
            option_name,
        });
    }
    if !requested.is_empty() {
        return Err(ApiError::bad_request("菜品规格已更新，请重新选择"));
    }
    Ok((price_cents, normalized))
}

fn to_cents(amount: f64) -> Result<i64, ApiError> {
    if !amount.is_finite() || amount < 0.0 || amount > (i64::MAX as f64 / 100.0) {
        return Err(ApiError::bad_request("菜品价格配置无效，请联系门店"));
    }
    let cents = amount * 100.0;
    let rounded = cents.round();
    if (cents - rounded).abs() > 0.0001 {
        return Err(ApiError::bad_request("菜品价格最多支持两位小数"));
    }
    Ok(rounded as i64)
}

fn calculate_earned_points(
    spend_cents: i64,
    spend_step_cents: i64,
    points_per_step: i64,
) -> Result<i64, ApiError> {
    if spend_cents < 0 || spend_step_cents <= 0 || points_per_step <= 0 {
        return Err(ApiError::service_unavailable("积分规则配置无效"));
    }

    let scaled_points = i128::from(spend_cents) * i128::from(points_per_step);
    let step = i128::from(spend_step_cents);
    let whole_points = scaled_points / step;
    let remainder = scaled_points % step;
    // Keep fractions up to 0.06; increment only when the remainder is greater than 6%.
    let rounded_points = whole_points + i128::from(remainder * 100 > step * 6);
    i64::try_from(rounded_points).map_err(|_| ApiError::conflict("积分超出可记录范围"))
}

fn make_order_number() -> String {
    let milliseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() % 10_000_000_000_000)
        .unwrap_or_default();
    let random = OsRng.next_u32();
    format!("HW{milliseconds:013}{random:08X}")
}

#[cfg(test)]
mod tests {
    use super::{apply_discount, calculate_earned_points, to_cents};

    #[test]
    fn currency_uses_integer_cents() {
        assert_eq!(to_cents(12.35).unwrap(), 1235);
        assert_eq!(to_cents(0.29).unwrap(), 29);
        assert!(to_cents(1.234).is_err());
    }

    #[test]
    fn member_discount_is_rounded_to_whole_cents() {
        assert_eq!(apply_discount(5_000, 0.98).unwrap(), 4_900);
        assert_eq!(apply_discount(1_299, 0.98).unwrap(), 1_273);
        assert_eq!(apply_discount(5_000, 1.0).unwrap(), 5_000);
        assert!(apply_discount(5_000, 1.2).is_err());
    }

    #[test]
    fn points_round_up_only_when_fraction_exceeds_six_percent() {
        assert_eq!(calculate_earned_points(2_389, 100, 1).unwrap(), 24);
        assert_eq!(calculate_earned_points(2_306, 100, 1).unwrap(), 23);
        assert_eq!(calculate_earned_points(2_307, 100, 1).unwrap(), 24);
        assert_eq!(calculate_earned_points(2_389, 100, 2).unwrap(), 48);
    }
}
