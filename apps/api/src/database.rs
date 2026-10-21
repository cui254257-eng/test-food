use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

pub(crate) async fn connect(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    initialize_schema(&pool).await?;
    remove_default_demo_data(&pool).await?;
    Ok(pool)
}

async fn initialize_schema(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS products (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(80) NOT NULL,
            category VARCHAR(40) NOT NULL DEFAULT '主食',
            description VARCHAR(500) NOT NULL DEFAULT '',
            specifications JSON NULL,
            price DOUBLE NOT NULL CHECK (price >= 0),
            stock BIGINT NOT NULL DEFAULT 0 CHECK (stock >= 0),
            status VARCHAR(16) NOT NULL DEFAULT 'active',
            discount_eligible BOOLEAN NOT NULL DEFAULT TRUE,
            image_mime_type VARCHAR(32) NULL,
            image_base64 LONGTEXT NULL,
            created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
            INDEX idx_products_status (status),
            INDEX idx_products_name (name),
            INDEX idx_products_category_status (category, status)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;
    let has_product_discount_eligible: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'products' AND COLUMN_NAME = 'discount_eligible'",
    )
    .fetch_one(pool)
    .await?;
    if has_product_discount_eligible == 0 {
        sqlx::query(
            "ALTER TABLE products ADD COLUMN discount_eligible BOOLEAN NOT NULL DEFAULT TRUE",
        )
        .execute(pool)
        .await?;
    }
    let has_product_image_mime_type: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'products' AND COLUMN_NAME = 'image_mime_type'",
    )
    .fetch_one(pool)
    .await?;
    if has_product_image_mime_type == 0 {
        sqlx::query("ALTER TABLE products ADD COLUMN image_mime_type VARCHAR(32) NULL")
            .execute(pool)
            .await?;
    }

    let has_product_image_base64: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'products' AND COLUMN_NAME = 'image_base64'",
    )
    .fetch_one(pool)
    .await?;
    if has_product_image_base64 == 0 {
        sqlx::query("ALTER TABLE products ADD COLUMN image_base64 LONGTEXT NULL")
            .execute(pool)
            .await?;
    }

    let has_product_category: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'products' AND COLUMN_NAME = 'category'",
    )
    .fetch_one(pool)
    .await?;
    if has_product_category == 0 {
        sqlx::query("ALTER TABLE products ADD COLUMN category VARCHAR(40) NOT NULL DEFAULT '主食'")
            .execute(pool)
            .await?;
    }

    // Nullable JSON preserves old rows; repository queries expose NULL as an empty option list.
    let has_product_specifications: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'products' AND COLUMN_NAME = 'specifications'",
    )
    .fetch_one(pool)
    .await?;
    if has_product_specifications == 0 {
        sqlx::query("ALTER TABLE products ADD COLUMN specifications JSON NULL")
            .execute(pool)
            .await?;
    }

    let has_product_category_index: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.STATISTICS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'products' AND INDEX_NAME = 'idx_products_category_status'",
    )
    .fetch_one(pool)
    .await?;
    if has_product_category_index == 0 {
        sqlx::query("CREATE INDEX idx_products_category_status ON products (category, status)")
            .execute(pool)
            .await?;
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS banners (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            title VARCHAR(80) NOT NULL DEFAULT '',
            mime_type VARCHAR(32) NOT NULL,
            image_base64 LONGTEXT NOT NULL,
            background_mime_type VARCHAR(32) NULL,
            background_image_base64 LONGTEXT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            sort_order INT NOT NULL DEFAULT 0,
            created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
            INDEX idx_banners_active_sort (is_active, sort_order, id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;

    for (column_name, definition) in [
        ("background_mime_type", "VARCHAR(32) NULL"),
        ("background_image_base64", "LONGTEXT NULL"),
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'banners' AND COLUMN_NAME = ?",
        )
        .bind(column_name)
        .fetch_one(pool)
        .await?;
        if exists == 0 {
            sqlx::query(&format!(
                "ALTER TABLE banners ADD COLUMN {column_name} {definition}"
            ))
            .execute(pool)
            .await?;
        }
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS home_features (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            label VARCHAR(40) NOT NULL,
            icon VARCHAR(64) NOT NULL,
            target_path VARCHAR(255) NOT NULL,
            sort_order INT NOT NULL DEFAULT 0,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
            INDEX idx_home_features_active_sort (is_active, sort_order, id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;

    // A single-row configuration for restaurant service modes, hours, address and delivery rules.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS restaurant_settings (
            id TINYINT NOT NULL PRIMARY KEY,
            restaurant_name VARCHAR(80) NOT NULL,
            announcement VARCHAR(255) NOT NULL DEFAULT '',
            announcement_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            business_hours VARCHAR(255) NOT NULL DEFAULT '',
            contact_phone VARCHAR(20) NOT NULL DEFAULT '',
            store_address VARCHAR(255) NOT NULL DEFAULT '',
            is_open BOOLEAN NOT NULL DEFAULT TRUE,
            dine_in_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            takeaway_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            delivery_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            delivery_radius_km DOUBLE NOT NULL DEFAULT 3,
            delivery_minimum_amount DOUBLE NOT NULL DEFAULT 0,
            delivery_fee DOUBLE NOT NULL DEFAULT 0,
            updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;

    let has_announcement_enabled: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'restaurant_settings' AND COLUMN_NAME = 'announcement_enabled'",
    )
    .fetch_one(pool)
    .await?;
    if has_announcement_enabled == 0 {
        sqlx::query("ALTER TABLE restaurant_settings ADD COLUMN announcement_enabled BOOLEAN NOT NULL DEFAULT TRUE")
            .execute(pool)
            .await?;
    }

    // Seed only once so restarting the API never overwrites the restaurant's edited settings.
    sqlx::query(
        "INSERT IGNORE INTO restaurant_settings \
         (id, restaurant_name, announcement, announcement_enabled, business_hours, contact_phone, store_address, is_open, \
          dine_in_enabled, takeaway_enabled, delivery_enabled, delivery_radius_km, delivery_minimum_amount, delivery_fee) \
         VALUES (1, '好味小馆', '', FALSE, '10:00-14:00, 17:00-22:00', '', '', TRUE, TRUE, TRUE, TRUE, 3, 0, 0)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE restaurant_settings SET announcement = '', announcement_enabled = FALSE \
         WHERE id = 1 AND announcement IN ('堂食、外带与外卖服务以门店营业状态为准', '堂食 / 自取 / 外卖')",
    )
    .execute(pool)
    .await?;
    sqlx::query("UPDATE restaurant_settings SET announcement_enabled = FALSE WHERE id = 1 AND TRIM(announcement) = ''")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS announcements (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            content VARCHAR(255) NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            sort_order INT NOT NULL DEFAULT 0,
            created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
            INDEX idx_announcements_active_sort (is_active, sort_order, id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;
    // Move the old single announcement into the new list once, then clear the legacy field.
    sqlx::query(
        "INSERT INTO announcements (content, is_active, sort_order)
         SELECT announcement, announcement_enabled, 0 FROM restaurant_settings
         WHERE id = 1 AND TRIM(announcement) <> ''
           AND NOT EXISTS (SELECT 1 FROM announcements)",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE restaurant_settings SET announcement = '', announcement_enabled = FALSE
         WHERE id = 1 AND TRIM(announcement) <> ''",
    )
    .execute(pool)
    .await?;

    // The OpenID is the customer's unique mini-program identity; session and address
    // ownership are always resolved server-side from the bearer token.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS customers (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            openid VARCHAR(128) NOT NULL UNIQUE,
            nickname VARCHAR(40) NOT NULL DEFAULT '',
            avatar_mime_type VARCHAR(32) NULL,
            avatar_base64 LONGTEXT NULL,
            created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;

    for (column_name, definition) in [
        ("nickname", "VARCHAR(40) NOT NULL DEFAULT ''"),
        ("avatar_mime_type", "VARCHAR(32) NULL"),
        ("avatar_base64", "LONGTEXT NULL"),
        ("points", "BIGINT NOT NULL DEFAULT 0"),
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'customers' AND COLUMN_NAME = ?",
        )
        .bind(column_name)
        .fetch_one(pool)
        .await?;
        if exists == 0 {
            sqlx::query(&format!(
                "ALTER TABLE customers ADD COLUMN {column_name} {definition}"
            ))
            .execute(pool)
            .await?;
        }
    }
    sqlx::query(
        "UPDATE customers SET nickname = CONCAT('微信用户', LPAD(FLOOR(RAND() * 1000000), 6, '0')) \
         WHERE TRIM(nickname) = ''",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS membership_settings (
            id TINYINT NOT NULL PRIMARY KEY,
            spend_step_cents BIGINT NOT NULL DEFAULT 100,
            points_per_step BIGINT NOT NULL DEFAULT 1,
            updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;
    sqlx::query("INSERT IGNORE INTO membership_settings (id, spend_step_cents, points_per_step) VALUES (1, 100, 1)")
        .execute(pool)
        .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS membership_tiers (
            level TINYINT NOT NULL PRIMARY KEY,
            required_points BIGINT NOT NULL,
            discount_rate DECIMAL(5,4) NOT NULL,
            updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;
    for (level, required_points, discount_rate) in [
        (1_i64, 0_i64, 1.0_f64),
        (2, 100, 0.98),
        (3, 300, 0.95),
        (4, 1000, 0.93),
        (5, 3000, 0.90),
    ] {
        sqlx::query("INSERT IGNORE INTO membership_tiers (level, required_points, discount_rate) VALUES (?, ?, ?)")
            .bind(level)
            .bind(required_points)
            .bind(discount_rate)
            .execute(pool)
            .await?;
    }

    // Store only a hash of the random bearer token; the raw token is returned once to the mini-program.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS customer_sessions (
            token_hash CHAR(64) NOT NULL PRIMARY KEY,
            customer_id BIGINT NOT NULL,
            expires_at DATETIME(3) NOT NULL,
            created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            INDEX idx_customer_sessions_customer (customer_id),
            INDEX idx_customer_sessions_expiry (expires_at),
            CONSTRAINT fk_customer_sessions_customer FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;

    // Existing 30-day sessions are shortened too; new sessions use the same 7-day policy.
    sqlx::query(
        "UPDATE customer_sessions SET expires_at = TIMESTAMPADD(DAY, 7, created_at) \
         WHERE expires_at > TIMESTAMPADD(DAY, 7, created_at)",
    )
    .execute(pool)
    .await?;

    // Address fields are personal data. Every API read/write is scoped by customer_id.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS customer_addresses (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            customer_id BIGINT NOT NULL,
            recipient_name VARCHAR(40) NOT NULL,
            phone VARCHAR(20) NOT NULL,
            province VARCHAR(50) NOT NULL,
            city VARCHAR(50) NOT NULL,
            district VARCHAR(50) NOT NULL,
            detail VARCHAR(255) NOT NULL,
            is_default BOOLEAN NOT NULL DEFAULT FALSE,
            created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
            INDEX idx_customer_addresses_owner (customer_id, is_default, id),
            CONSTRAINT fk_customer_addresses_customer FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;

    // Orders and item snapshots are separate from the live menu so later menu edits do not rewrite receipts.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS restaurant_orders (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            order_number VARCHAR(32) NOT NULL UNIQUE,
            customer_id BIGINT NOT NULL,
            request_key VARCHAR(64) NOT NULL,
            fulfillment_mode VARCHAR(16) NOT NULL,
            table_number VARCHAR(20) NULL,
            delivery_recipient_name VARCHAR(40) NULL,
            delivery_phone VARCHAR(20) NULL,
            delivery_address VARCHAR(500) NULL,
            status VARCHAR(24) NOT NULL,
            payment_status VARCHAR(16) NOT NULL,
            expires_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            stock_released BOOLEAN NOT NULL DEFAULT FALSE,
            subtotal_cents BIGINT NOT NULL,
            discount_cents BIGINT NOT NULL DEFAULT 0,
            delivery_fee_cents BIGINT NOT NULL,
            total_cents BIGINT NOT NULL,
            points_earned BIGINT NOT NULL DEFAULT 0,
            created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
            UNIQUE KEY uq_restaurant_orders_request (customer_id, request_key),
            INDEX idx_restaurant_orders_customer (customer_id, created_at),
            INDEX idx_restaurant_orders_expiry (status, payment_status, stock_released, expires_at),
            CONSTRAINT fk_restaurant_orders_customer FOREIGN KEY (customer_id) REFERENCES customers(id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;
    for (column_name, definition) in [
        ("discount_cents", "BIGINT NOT NULL DEFAULT 0"),
        ("points_earned", "BIGINT NOT NULL DEFAULT 0"),
        (
            "expires_at",
            "DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3)",
        ),
        ("stock_released", "BOOLEAN NOT NULL DEFAULT FALSE"),
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'restaurant_orders' AND COLUMN_NAME = ?",
        )
        .bind(column_name)
        .fetch_one(pool)
        .await?;
        if exists == 0 {
            sqlx::query(&format!(
                "ALTER TABLE restaurant_orders ADD COLUMN {column_name} {definition}"
            ))
            .execute(pool)
            .await?;
        }
    }
    let has_order_expiry_index: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.STATISTICS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'restaurant_orders' AND INDEX_NAME = 'idx_restaurant_orders_expiry'",
    )
    .fetch_one(pool)
    .await?;
    if has_order_expiry_index == 0 {
        sqlx::query("CREATE INDEX idx_restaurant_orders_expiry ON restaurant_orders (status, payment_status, stock_released, expires_at)")
            .execute(pool)
            .await?;
    }
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS restaurant_order_items (
            id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            order_id BIGINT NOT NULL,
            product_id BIGINT NOT NULL,
            product_name VARCHAR(80) NOT NULL,
            category VARCHAR(40) NOT NULL,
            unit_price_cents BIGINT NOT NULL,
            quantity BIGINT NOT NULL,
            line_total_cents BIGINT NOT NULL,
            discount_cents BIGINT NOT NULL DEFAULT 0,
            discount_eligible BOOLEAN NOT NULL DEFAULT TRUE,
            selected_options JSON NOT NULL,
            INDEX idx_restaurant_order_items_order (order_id, id),
            CONSTRAINT fk_restaurant_order_items_order FOREIGN KEY (order_id) REFERENCES restaurant_orders(id) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;

    for (column_name, definition) in [
        ("discount_cents", "BIGINT NOT NULL DEFAULT 0"),
        ("discount_eligible", "BOOLEAN NOT NULL DEFAULT TRUE"),
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'restaurant_order_items' AND COLUMN_NAME = ?",
        )
        .bind(column_name)
        .fetch_one(pool)
        .await?;
        if exists == 0 {
            sqlx::query(&format!(
                "ALTER TABLE restaurant_order_items ADD COLUMN {column_name} {definition}"
            ))
            .execute(pool)
            .await?;
        }
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS app_migrations (
            migration_key VARCHAR(100) NOT NULL PRIMARY KEY,
            applied_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci",
    )
    .execute(pool)
    .await?;

    let has_banner_updated_at: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'banners' AND COLUMN_NAME = 'updated_at'",
    )
    .fetch_one(pool)
    .await?;
    if has_banner_updated_at == 0 {
        sqlx::query(
            "ALTER TABLE banners ADD COLUMN updated_at DATETIME(3) NOT NULL \
             DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3)",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn remove_default_demo_data(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    const MIGRATION_KEY: &str = "2026-10-remove-default-demo-data-v2";

    let mut tx = pool.begin().await?;
    let already_applied: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM app_migrations WHERE migration_key = ?)")
            .bind(MIGRATION_KEY)
            .fetch_one(&mut *tx)
            .await?;

    if already_applied {
        tx.commit().await?;
        return Ok(());
    }

    // Match known seeded values and leave edited/uploaded menu items alone.
    let removed_products = sqlx::query(
        "DELETE FROM products WHERE image_mime_type IS NULL AND image_base64 IS NULL AND (
            (name = '招牌红烧牛肉饭' AND category = '主食盖饭' AND description = '慢炖牛腩，配米饭和时蔬。' AND price = 32 AND stock = 60 AND status = 'active') OR
            (name = '香煎鸡腿盖饭' AND category = '主食盖饭' AND description = '香煎鸡腿配米饭。' AND price = 28 AND stock = 45 AND status = 'active') OR
            (name = '番茄鸡蛋面' AND category = '面食' AND description = '番茄鸡蛋汤面。' AND price = 22 AND stock = 40 AND status = 'active') OR
            (name = '手打柠檬茶' AND category = '饮品' AND description = '鲜柠檬现捶。' AND price = 12 AND stock = 80 AND status = 'active') OR
            (name = '午间双人套餐' AND category = '套餐' AND description = '两份主食和两杯饮品。' AND price = 58 AND stock = 30 AND status = 'active') OR
            (name = '杨枝甘露' AND category = '甜品' AND description = '芒果、西柚和椰奶。' AND price = 18 AND stock = 0 AND status = 'inactive') OR
            (name = '每日坚果礼盒' AND description = '精选每日坚果，独立小包装，方便随身携带。') OR
            (name = '手冲咖啡组合' AND description = '包含三款风味咖啡豆，开启轻松的早晨。') OR
            (name = '陶瓷马克杯' AND description = '简约釉面陶瓷杯，适合办公室与居家使用。')
        )",
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    let removed_features = sqlx::query(
        "DELETE FROM home_features WHERE is_active = TRUE AND (
            (label = '人气热卖' AND icon = 'hot.svg' AND target_path = '/pages/products/index?q=%E7%89%9B%E8%82%89%E9%A5%AD') OR
            (label = '招牌盖饭' AND icon = 'home.svg' AND target_path = '/pages/products/index?q=%E7%9B%96%E9%A5%AD') OR
            (label = '面食小吃' AND icon = 'snacks.svg' AND target_path = '/pages/products/index?q=%E9%9D%A2') OR
            (label = '现制饮品' AND icon = 'coffee.svg' AND target_path = '/pages/products/index?q=%E8%8C%B6') OR
            (label = '今日上新' AND icon = 'new.svg' AND target_path = '/pages/products/index?sort=newest') OR
            (label = '优惠套餐' AND icon = 'price.svg' AND target_path = '/pages/products/index?q=%E5%A5%97%E9%A4%90') OR
            (label = '点餐篮' AND icon = 'cart.svg' AND target_path = '/pages/cart/index') OR
            (label = '全部菜品' AND icon = 'all.svg' AND target_path = '/pages/products/index')
        )",
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    sqlx::query("INSERT IGNORE INTO app_migrations (migration_key) VALUES (?)")
        .bind(MIGRATION_KEY)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    tracing::info!(
        removed_products,
        removed_features,
        "Removed seeded demo catalog data"
    );
    Ok(())
}
