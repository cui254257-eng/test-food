use std::{
    collections::{BTreeMap, HashMap},
    time::{SystemTime, UNIX_EPOCH},
};

use rand::{RngCore, rngs::OsRng};
use sqlx::{FromRow, MySqlPool, types::Json};

use crate::{common::ApiError, modules::products::ProductSpecificationGroup};

use super::{
    dto::{OrderInput, OrderItemInput, SpecificationSelection},
    entity::Order,
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
    selections: Vec<SpecificationSelection>,
}

impl OrdersRepository {
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub(crate) async fn create(
        &self,
        customer_id: i64,
        input: OrderInput,
    ) -> Result<Order, ApiError> {
        let mut tx = self.pool.begin().await?;

        // Serialize submissions per customer; this makes request-key retries safe and prevents duplicate stock deductions.
        let customer_exists: Option<i64> =
            sqlx::query_scalar("SELECT id FROM customers WHERE id = ? FOR UPDATE")
                .bind(customer_id)
                .fetch_optional(&mut *tx)
                .await?;
        if customer_exists.is_none() {
            return Err(ApiError::unauthorized("登录已过期，请重新登录"));
        }

        if let Some(existing) = sqlx::query_as::<_, Order>(
            "SELECT order_number, fulfillment_mode, status, payment_status, subtotal_cents, \
             delivery_fee_cents, total_cents FROM restaurant_orders \
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
                "SELECT id, name, category, COALESCE(specifications, JSON_ARRAY()) AS specifications, price, stock, status \
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
        let mut snapshots = Vec::with_capacity(input.items.len());
        for item in &input.items {
            let product = products
                .get(&item.product_id)
                .expect("all requested products were loaded under lock");
            let base_price = to_cents(product.price)?;
            let (unit_price_cents, selections) =
                resolve_selections(&product.specifications.0, item, base_price)?;
            let line_total_cents = unit_price_cents
                .checked_mul(item.quantity)
                .ok_or_else(|| ApiError::bad_request("订单金额超出范围"))?;
            subtotal_cents = subtotal_cents
                .checked_add(line_total_cents)
                .ok_or_else(|| ApiError::bad_request("订单金额超出范围"))?;
            snapshots.push(ItemSnapshot {
                product_id: product.id,
                product_name: product.name.clone(),
                category: product.category.clone(),
                unit_price_cents,
                quantity: item.quantity,
                line_total_cents,
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
        let total_cents = subtotal_cents
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
              expires_at, subtotal_cents, delivery_fee_cents, total_cents) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'pending_payment', 'unpaid', \
                     DATE_ADD(CURRENT_TIMESTAMP(3), INTERVAL 15 MINUTE), ?, ?, ?)",
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
        .bind(delivery_fee_cents)
        .bind(total_cents)
        .execute(&mut *tx)
        .await?;
        let order_id = result.last_insert_id() as i64;

        for snapshot in &snapshots {
            sqlx::query(
                "INSERT INTO restaurant_order_items \
                 (order_id, product_id, product_name, category, unit_price_cents, quantity, line_total_cents, selected_options) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(order_id)
            .bind(snapshot.product_id)
            .bind(&snapshot.product_name)
            .bind(&snapshot.category)
            .bind(snapshot.unit_price_cents)
            .bind(snapshot.quantity)
            .bind(snapshot.line_total_cents)
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
            delivery_fee_cents,
            total_cents,
        })
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
    use super::to_cents;

    #[test]
    fn currency_uses_integer_cents() {
        assert_eq!(to_cents(12.35).unwrap(), 1235);
        assert_eq!(to_cents(0.29).unwrap(), 29);
        assert!(to_cents(1.234).is_err());
    }
}
