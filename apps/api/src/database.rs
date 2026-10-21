use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

pub(crate) async fn connect(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    initialize_schema(&pool).await?;
    seed_products(&pool).await?;
    seed_home_features(&pool).await?;
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
            delivery_fee_cents BIGINT NOT NULL,
            total_cents BIGINT NOT NULL,
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
            selected_options JSON NOT NULL,
            INDEX idx_restaurant_order_items_order (order_id, id),
            CONSTRAINT fk_restaurant_order_items_order FOREIGN KEY (order_id) REFERENCES restaurant_orders(id) ON DELETE CASCADE
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

async fn seed_products(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products")
        .fetch_one(pool)
        .await?;
    if count == 0 {
        sqlx::query(
            "INSERT INTO products (name, category, description, price, stock, status) VALUES
             ('招牌红烧牛肉饭', '主食盖饭', '慢炖牛腩，配米饭和时蔬。', 32.00, 60, 'active'),
             ('香煎鸡腿盖饭', '主食盖饭', '香煎鸡腿配米饭。', 28.00, 45, 'active'),
             ('番茄鸡蛋面', '面食', '番茄鸡蛋汤面。', 22.00, 40, 'active'),
             ('手打柠檬茶', '饮品', '鲜柠檬现捶。', 12.00, 80, 'active'),
             ('午间双人套餐', '套餐', '两份主食和两杯饮品。', 58.00, 30, 'active'),
             ('杨枝甘露', '甜品', '芒果、西柚和椰奶。', 18.00, 0, 'inactive')",
        )
        .execute(pool)
        .await?;
    }

    // Convert only the repository's original sample merchandise rows; store-entered menu items are not overwritten.
    sqlx::query(
        "UPDATE products SET name = '招牌红烧牛肉饭', category = '主食盖饭', description = '慢炖牛腩，配米饭和时蔬。', price = 32, stock = 60, status = 'active' \
         WHERE name = '每日坚果礼盒' AND description = '精选每日坚果，独立小包装，方便随身携带。'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE products SET name = '手打柠檬茶', category = '饮品', description = '鲜柠檬现捶。', price = 12, stock = 80, status = 'active' \
         WHERE name = '手冲咖啡组合' AND description = '包含三款风味咖啡豆，开启轻松的早晨。'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE products SET name = '杨枝甘露', category = '甜品', description = '芒果、西柚和椰奶。', price = 18, stock = 0, status = 'inactive' \
         WHERE name = '陶瓷马克杯' AND description = '简约釉面陶瓷杯，适合办公室与居家使用。'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE products SET category = '主食盖饭' WHERE name = '香煎鸡腿盖饭' AND category = '主食'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE products SET description = '香煎鸡腿配米饭。' \
         WHERE name = '香煎鸡腿盖饭' AND description = '外酥里嫩鸡腿排，搭配每日鲜蔬和热米饭。'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE products SET category = '面食' WHERE name = '番茄鸡蛋面' AND category = '主食'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE products SET description = '番茄鸡蛋汤面。' \
         WHERE name = '番茄鸡蛋面' AND description = '酸甜番茄汤底，现煮面条，暖胃家常味。'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE products SET description = '两份主食和两杯饮品。' \
         WHERE name = '午间双人套餐' AND description = '两份主食搭配两杯饮品，适合工作日午餐分享。'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE products SET specifications = JSON_ARRAY( \
            JSON_OBJECT('name', '份量', 'required', TRUE, 'options', JSON_ARRAY( \
                JSON_OBJECT('name', '标准份', 'price_delta', 0), JSON_OBJECT('name', '加大份', 'price_delta', 5))), \
            JSON_OBJECT('name', '辣度', 'required', TRUE, 'options', JSON_ARRAY( \
                JSON_OBJECT('name', '不辣', 'price_delta', 0), JSON_OBJECT('name', '微辣', 'price_delta', 0), JSON_OBJECT('name', '中辣', 'price_delta', 0)))) \
         WHERE name = '招牌红烧牛肉饭' AND specifications IS NULL",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE products SET specifications = JSON_ARRAY( \
            JSON_OBJECT('name', '糖度', 'required', TRUE, 'options', JSON_ARRAY( \
                JSON_OBJECT('name', '正常糖', 'price_delta', 0), JSON_OBJECT('name', '少糖', 'price_delta', 0), JSON_OBJECT('name', '无糖', 'price_delta', 0))), \
            JSON_OBJECT('name', '冰量', 'required', TRUE, 'options', JSON_ARRAY( \
                JSON_OBJECT('name', '正常冰', 'price_delta', 0), JSON_OBJECT('name', '少冰', 'price_delta', 0), JSON_OBJECT('name', '去冰', 'price_delta', 0)))) \
         WHERE name = '手打柠檬茶' AND specifications IS NULL",
    )
    .execute(pool)
    .await?;
    // Add missing starter dishes on an existing demo database without replacing any store-managed row.
    sqlx::query(
        "INSERT INTO products (name, category, description, price, stock, status) \
         SELECT '香煎鸡腿盖饭', '主食盖饭', '香煎鸡腿配米饭。', 28, 45, 'active' FROM DUAL \
         WHERE NOT EXISTS (SELECT 1 FROM products WHERE name = '香煎鸡腿盖饭')",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO products (name, category, description, price, stock, status) \
         SELECT '番茄鸡蛋面', '面食', '番茄鸡蛋汤面。', 22, 40, 'active' FROM DUAL \
         WHERE NOT EXISTS (SELECT 1 FROM products WHERE name = '番茄鸡蛋面')",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO products (name, category, description, price, stock, status) \
         SELECT '午间双人套餐', '套餐', '两份主食和两杯饮品。', 58, 30, 'active' FROM DUAL \
         WHERE NOT EXISTS (SELECT 1 FROM products WHERE name = '午间双人套餐')",
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn seed_home_features(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM home_features")
        .fetch_one(pool)
        .await?;
    if count == 0 {
        sqlx::query(
            "INSERT INTO home_features (label, icon, target_path, sort_order, is_active) VALUES
             ('人气热卖', 'hot.svg', '/pages/products/index?q=%E7%89%9B%E8%82%89%E9%A5%AD', 10, TRUE),
             ('招牌盖饭', 'home.svg', '/pages/products/index?q=%E7%9B%96%E9%A5%AD', 20, TRUE),
             ('面食小吃', 'snacks.svg', '/pages/products/index?q=%E9%9D%A2', 30, TRUE),
             ('现制饮品', 'coffee.svg', '/pages/products/index?q=%E8%8C%B6', 40, TRUE),
             ('今日上新', 'new.svg', '/pages/products/index?sort=newest', 50, TRUE),
             ('优惠套餐', 'price.svg', '/pages/products/index?q=%E5%A5%97%E9%A4%90', 60, TRUE),
             ('点餐篮', 'cart.svg', '/pages/cart/index', 70, TRUE),
             ('全部菜品', 'all.svg', '/pages/products/index', 80, TRUE)",
        )
        .execute(pool)
        .await?;
    }

    // Update only the original demo shortcuts; user-created/edited shortcuts are left untouched.
    sqlx::query(
        "UPDATE home_features SET label = '人气热卖', target_path = '/pages/products/index?q=%E7%89%9B%E8%82%89%E9%A5%AD' WHERE label = '人气榜单' AND target_path LIKE '/pages/products/index%'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE home_features SET label = '招牌盖饭', target_path = '/pages/products/index?q=%E7%9B%96%E9%A5%AD' WHERE label = '家居好物'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE home_features SET label = '面食小吃', target_path = '/pages/products/index?q=%E9%9D%A2' WHERE label = '坚果零食'",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE home_features SET label = '现制饮品', target_path = '/pages/products/index?q=%E8%8C%B6' WHERE label = '咖啡茶饮'",
    )
    .execute(pool)
    .await?;
    sqlx::query("UPDATE home_features SET label = '全部菜品' WHERE label = '全部商品'")
        .execute(pool)
        .await?;
    sqlx::query("UPDATE home_features SET label = '点餐篮' WHERE label = '购物车'")
        .execute(pool)
        .await?;
    sqlx::query("UPDATE home_features SET label = '今日上新' WHERE label = '新品上架'")
        .execute(pool)
        .await?;
    sqlx::query("UPDATE home_features SET label = '优惠套餐' WHERE label = '今日好价'")
        .execute(pool)
        .await?;
    sqlx::query(
        "UPDATE home_features SET target_path = '/pages/products/index?q=%E5%A5%97%E9%A4%90' WHERE label = '优惠套餐' AND target_path LIKE '%sort=price%'",
    )
    .execute(pool)
    .await?;
    Ok(())
}
