//! Integration tests for GraphQL schema.

use async_graphql::{Request, Variables, Name, Value};
use auth::{SqliteUserRepository, UserRepository};
use graphql::{build_schema, GraphQLContext};
use persistence::{DbPool, DbPoolConfig};
use uuid::Uuid;

/// Helper to convert serde_json::Value to async_graphql::Value
fn json_to_gql_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                Value::Number((u as i64).into())
            } else {
                // For floats, convert to string representation
                Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::List(arr.into_iter().map(json_to_gql_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let map = obj
                .into_iter()
                .map(|(k, v)| (Name::new(k), json_to_gql_value(v)))
                .collect();
            Value::Object(map)
        }
    }
}

async fn create_test_pool() -> DbPool {
    let config = DbPoolConfig {
        database_url: "sqlite::memory:".to_string(),
        max_connections: 1,
        connection_timeout_secs: 5,
    };

    let pool = DbPool::new(&config).await.unwrap();

    // Run migrations
    sqlx::migrate!("../../../migrations")
        .run(pool.inner())
        .await
        .unwrap();

    pool
}

#[tokio::test]
async fn test_health_query() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let context = GraphQLContext::new(pool.clone());

    let query = r#"
        query {
            health {
                status
                version
                database
            }
        }
    "#;

    let response = schema
        .execute(Request::new(query).data(context))
        .await;

    assert!(response.errors.is_empty());
    let data = response.data.into_json().unwrap();
    assert_eq!(data["health"]["status"], "healthy");
    assert_eq!(data["health"]["database"], "connected");

    pool.close().await;
}

#[tokio::test]
async fn test_login_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let context = GraphQLContext::new(pool.clone());

    // Create a test user
    let user_repo = SqliteUserRepository::new(pool.clone());
    user_repo
        .create("testuser", "test@example.com", "password123")
        .await
        .unwrap();

    let mutation = r#"
        mutation Login($username: String!, $password: String!) {
            login(username: $username, password: $password) {
                user {
                    username
                    email
                }
                token
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(async_graphql::Name::new("username"), async_graphql::Value::from("testuser"));
    variables.insert(async_graphql::Name::new("password"), async_graphql::Value::from("password123"));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty());
    let data = response.data.into_json().unwrap();
    assert_eq!(data["login"]["user"]["username"], "testuser");
    assert_eq!(data["login"]["user"]["email"], "test@example.com");
    assert!(data["login"]["token"].as_str().unwrap().len() > 0);

    pool.close().await;
}

#[tokio::test]
async fn test_current_user_query_without_auth() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let context = GraphQLContext::new(pool.clone());

    let query = r#"
        query {
            currentUser {
                username
            }
        }
    "#;

    let response = schema
        .execute(Request::new(query).data(context))
        .await;

    assert!(!response.errors.is_empty());
    assert!(response.errors[0].message.contains("Authentication required"));

    pool.close().await;
}

#[tokio::test]
async fn test_current_user_query_with_auth() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create a test user
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("testuser", "test@example.com", "password123")
        .await
        .unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user);

    let query = r#"
        query {
            currentUser {
                username
                email
                isActive
            }
        }
    "#;

    let response = schema
        .execute(Request::new(query).data(context))
        .await;

    assert!(response.errors.is_empty());
    let data = response.data.into_json().unwrap();
    assert_eq!(data["currentUser"]["username"], "testuser");
    assert_eq!(data["currentUser"]["email"], "test@example.com");
    assert_eq!(data["currentUser"]["isActive"], true);

    pool.close().await;
}

#[tokio::test]
async fn test_reserve_inventory_partial_allocation() {
    let pool = create_test_pool().await;
    // Seed item/location/position
    let sku = "GQL-PARTIAL";
    let loc = Uuid::new_v4().to_string();
    // item
    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, 'EN', 'ZH', 'part', 'none', datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(sku)
    .execute(pool.inner())
    .await
    .unwrap();
    // location
    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, 'WH', 'A', '001', datetime('now'))"#,
    )
    .bind(&loc)
    .execute(pool.inner())
    .await
    .unwrap();
    // position 100 available
    sqlx::query(
        r#"INSERT INTO inventory_positions
           (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
           VALUES (?, ?, ?, 'available', NULL, 100, 0, 100, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(sku)
    .bind(&loc)
    .execute(pool.inner())
    .await
    .unwrap();

    // Create a user and auth context
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("invuser", "inv@example.com", "pw123")
        .await
        .unwrap();
    let schema = build_schema();
    let context = GraphQLContext::with_user(pool.clone(), user);

    let mutation = r#"
        mutation Reserve($sku: String!, $loc: String!, $qty: Int!, $partial: Boolean!) {
            reserveInventory(sku: $sku, locationId: $loc, quantity: $qty, allowPartial: $partial) {
                movementId
                allocated
            }
        }
    "#;
    let mut vars = Variables::default();
    vars.insert(Name::new("sku"), Value::from(sku));
    vars.insert(Name::new("loc"), Value::from(loc.clone()));
    vars.insert(Name::new("qty"), Value::from(150));
    vars.insert(Name::new("partial"), Value::from(true));

    let resp = schema
        .execute(Request::new(mutation).variables(vars).data(context))
        .await;
    assert!(resp.errors.is_empty(), "errors: {:?}", resp.errors);
    let data = resp.data.into_json().unwrap();
    assert_eq!(data["reserveInventory"]["allocated"], 100);

    // Verify DB updated
    let (on_hand, reserved, available): (i64, i64, i64) = sqlx::query_as(
        "SELECT quantity_on_hand, quantity_reserved, quantity_available FROM inventory_positions WHERE sku = ? AND location_id = ?",
    )
    .bind(sku)
    .bind(&loc)
    .fetch_one(pool.inner())
    .await
    .unwrap();
    assert_eq!(on_hand, 100);
    assert_eq!(reserved, 100);
    assert_eq!(available, 0);

    pool.close().await;
}

#[tokio::test]
async fn test_reserve_inventory_conflict_and_partial() {
    let pool = create_test_pool().await;
    let sku = "GQL-CONFLICT";
    let loc = Uuid::new_v4().to_string();
    // seed
    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, 'EN', 'ZH', 'part', 'none', datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(sku)
    .execute(pool.inner())
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, 'WH', 'A', '001', datetime('now'))"#,
    )
    .bind(&loc)
    .execute(pool.inner())
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO inventory_positions
           (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
           VALUES (?, ?, ?, 'available', NULL, 100, 0, 100, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(sku)
    .bind(&loc)
    .execute(pool.inner())
    .await
    .unwrap();

    // user
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("invuser2", "inv2@example.com", "pw123")
        .await
        .unwrap();
    let schema = build_schema();
    let context = GraphQLContext::with_user(pool.clone(), user);

    // first reserve 80
    let mut vars = Variables::default();
    vars.insert(Name::new("sku"), Value::from(sku));
    vars.insert(Name::new("loc"), Value::from(loc.clone()));
    vars.insert(Name::new("qty"), Value::from(80));
    vars.insert(Name::new("partial"), Value::from(false));
    let resp1 = schema
        .execute(Request::new(
            r#"mutation($sku:String!,$loc:String!,$qty:Int!,$partial:Boolean!){reserveInventory(sku:$sku,locationId:$loc,quantity:$qty,allowPartial:$partial){allocated movementId}}"#
        ).variables(vars.clone()).data(context.clone()))
        .await;
    assert!(resp1.errors.is_empty());

    // second reserve 30 without partial -> should error
    let mut vars2 = Variables::default();
    vars2.insert(Name::new("sku"), Value::from(sku));
    vars2.insert(Name::new("loc"), Value::from(loc.clone()));
    vars2.insert(Name::new("qty"), Value::from(30));
    vars2.insert(Name::new("partial"), Value::from(false));
    let resp2 = schema
        .execute(Request::new(
            r#"mutation($sku:String!,$loc:String!,$qty:Int!,$partial:Boolean!){reserveInventory(sku:$sku,locationId:$loc,quantity:$qty,allowPartial:$partial){allocated movementId}}"#
        ).variables(vars2).data(context.clone()))
        .await;
    assert!(!resp2.errors.is_empty());

    // third reserve 30 with partial -> should allocate remaining 20
    let mut vars3 = Variables::default();
    vars3.insert(Name::new("sku"), Value::from(sku));
    vars3.insert(Name::new("loc"), Value::from(loc.clone()));
    vars3.insert(Name::new("qty"), Value::from(30));
    vars3.insert(Name::new("partial"), Value::from(true));
    let resp3 = schema
        .execute(Request::new(
            r#"mutation($sku:String!,$loc:String!,$qty:Int!,$partial:Boolean!){reserveInventory(sku:$sku,locationId:$loc,quantity:$qty,allowPartial:$partial){allocated movementId}}"#
        ).variables(vars3).data(context.clone()))
        .await;
    assert!(resp3.errors.is_empty());
    let data3 = resp3.data.into_json().unwrap();
    assert_eq!(data3["reserveInventory"]["allocated"], 20);

    pool.close().await;
}

#[tokio::test]
async fn test_create_user_mutation_requires_permission() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create a test user without admin permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("testuser", "test@example.com", "password123")
        .await
        .unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user);

    let mutation = r#"
        mutation {
            createUser(username: "newuser", email: "new@example.com", password: "pass123") {
                username
            }
        }
    "#;

    let response = schema
        .execute(Request::new(mutation).data(context))
        .await;

    assert!(!response.errors.is_empty());
    assert!(response.errors[0].message.contains("Permission denied"));

    pool.close().await;
}

#[tokio::test]
async fn test_user_roles_and_permissions() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create a test user
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("testuser", "test@example.com", "password123")
        .await
        .unwrap();

    // Assign admin role
    let admin_role = auth::find_role_by_name(&pool, "admin").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, admin_role.id, None).await.unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user);

    let query = r#"
        query {
            currentUser {
                username
                roles {
                    name
                    description
                }
                permissions {
                    name
                    resource
                    action
                }
            }
        }
    "#;

    let response = schema
        .execute(Request::new(query).data(context))
        .await;

    assert!(response.errors.is_empty());
    let data = response.data.into_json().unwrap();
    assert_eq!(data["currentUser"]["username"], "testuser");
    
    // Check that user has admin role
    let roles = data["currentUser"]["roles"].as_array().unwrap();
    assert!(roles.iter().any(|r| r["name"] == "admin"));

    // Check that user has permissions
    let permissions = data["currentUser"]["permissions"].as_array().unwrap();
    assert!(permissions.len() > 0);
    assert!(permissions.iter().any(|p| p["name"] == "admin.users"));

    pool.close().await;
}

async fn create_cycle_count_tables(pool: &DbPool) {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS cycle_count_sessions (
            id TEXT PRIMARY KEY NOT NULL,
            abc_class TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL,
            completed_at TEXT
        )"#,
    )
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS cycle_count_lines (
            id TEXT PRIMARY KEY NOT NULL,
            session_id TEXT NOT NULL,
            sku TEXT NOT NULL,
            location_id TEXT NOT NULL,
            expected_qty INTEGER NOT NULL,
            counted_qty INTEGER NOT NULL,
            variance INTEGER NOT NULL,
            status TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES cycle_count_sessions(id) ON DELETE CASCADE
        )"#,
    )
    .execute(pool.inner())
    .await
    .unwrap();
}

#[tokio::test]
async fn test_cycle_count_flow_partial_and_audit() {
    let pool = create_test_pool().await;
    create_cycle_count_tables(&pool).await;

    // seed item/location/position (available 90)
    let sku = "SKU-CC-1";
    let loc = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, 'EN', 'ZH', 'part', 'none', datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(sku)
    .execute(pool.inner())
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, 'WH', 'C', '010', datetime('now'))"#,
    )
    .bind(&loc)
    .execute(pool.inner())
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO inventory_positions
           (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
           VALUES (?, ?, ?, 'available', NULL, 100, 10, 90, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(sku)
    .bind(&loc)
    .execute(pool.inner())
    .await
    .unwrap();

    // user
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("ccuser", "cc@example.com", "pw123")
        .await
        .unwrap();
    let schema = build_schema();
    let context = GraphQLContext::with_user(pool.clone(), user);

    // start session
    let start_mut = r#"mutation{ startCycleCount(abcClass:"A"){ id abcClass status } }"#;
    let resp = schema.execute(Request::new(start_mut).data(context.clone())).await;
    assert!(resp.errors.is_empty(), "{:?}", resp.errors);
    let data = resp.data.into_json().unwrap();
    let session_id = data["startCycleCount"]["id"].as_str().unwrap().to_string();

    // record count counted=85, expected=90 -> variance -5
    let mut vars = Variables::default();
    vars.insert(Name::new("sid"), Value::from(session_id.clone()));
    vars.insert(Name::new("sku"), Value::from(sku));
    vars.insert(Name::new("loc"), Value::from(loc.clone()));
    vars.insert(Name::new("cnt"), Value::from(85));
    let rec_mut = r#"mutation($sid:String!,$sku:String!,$loc:String!,$cnt:Int!){
        recordCycleCount(sessionId:$sid, sku:$sku, locationId:$loc, countedQty:$cnt){ id sessionId sku locationId expectedQty countedQty variance status }
    }"#;
    let resp2 = schema
        .execute(Request::new(rec_mut).variables(vars).data(context.clone()))
        .await;
    assert!(resp2.errors.is_empty(), "{:?}", resp2.errors);
    let data2 = resp2.data.into_json().unwrap();
    assert_eq!(data2["recordCycleCount"]["variance"], -5);

    // query session with lines
    let mut qvars = Variables::default();
    qvars.insert(Name::new("sid"), Value::from(session_id.clone()));
    let q = r#"query($sid:String!){ cycleCountSession(id:$sid){ id status lines{ sku locationId expectedQty countedQty variance } } }"#;
    let resp3 = schema
        .execute(Request::new(q).variables(qvars).data(context.clone()))
        .await;
    assert!(resp3.errors.is_empty(), "{:?}", resp3.errors);
    let data3 = resp3.data.into_json().unwrap();
    assert_eq!(data3["cycleCountSession"]["lines"][0]["variance"], -5);

    // complete session
    let mut cvars = Variables::default();
    cvars.insert(Name::new("sid"), Value::from(session_id.clone()));
    let comp_mut = r#"mutation($sid:String!){ completeCycleCount(sessionId:$sid) }"#;
    let resp4 = schema
        .execute(Request::new(comp_mut).variables(cvars).data(context.clone()))
        .await;
    assert!(resp4.errors.is_empty(), "{:?}", resp4.errors);
    let data4 = resp4.data.into_json().unwrap();
    assert_eq!(data4["completeCycleCount"], true);

    // verify audit event exists
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_events WHERE resource_id = ?")
        .bind(&session_id)
        .fetch_one(pool.inner())
        .await
        .unwrap();
    assert!(count >= 2); // start + record at least

    pool.close().await;
}

// ============================================================================
// Inventory Domain Tests (ADV-GQL-004)
// ============================================================================

/// Helper to create a user with warehouse.inventory permission
async fn create_inventory_user(pool: &DbPool) -> auth::User {
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("invuser", "inv@example.com", "password123")
        .await
        .unwrap();

    // Assign admin role (has all permissions including warehouse.inventory and warehouse.receiving)
    let role = auth::find_role_by_name(pool, "admin")
        .await
        .unwrap()
        .unwrap();
    auth::assign_role(pool, user.id, role.id, None)
        .await
        .unwrap();

    user
}

#[tokio::test]
async fn test_create_item_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    let mutation = r#"
        mutation CreateItem($input: CreateItemInput!) {
            createItem(input: $input) {
                id
                sku
                name
                nameZh
                itemType
                trackingMode
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(
        Name::new("input"),
        json_to_gql_value(serde_json::json!({
            "sku": "TEST-SKU-001",
            "name": "Test Item",
            "nameZh": "测试物品",
            "description": "Test description",
            "descriptionZh": "测试描述",
            "itemType": "part",
            "trackingMode": "none",
            "manufacturerPartNo": "MPN-001"
        })),
    );

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["createItem"]["sku"], "TEST-SKU-001");
    assert_eq!(data["createItem"]["name"], "Test Item");
    assert_eq!(data["createItem"]["nameZh"], "测试物品");
    assert_eq!(data["createItem"]["itemType"], "part");

    pool.close().await;
}

#[tokio::test]
async fn test_create_location_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    let mutation = r#"
        mutation CreateLocation($input: CreateLocationInput!) {
            createLocation(input: $input) {
                id
                warehouse
                zone
                bin
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(
        Name::new("input"),
        json_to_gql_value(serde_json::json!({
            "warehouse": "WH-01",
            "zone": "A",
            "bin": "001"
        })),
    );

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["createLocation"]["warehouse"], "WH-01");
    assert_eq!(data["createLocation"]["zone"], "A");
    assert_eq!(data["createLocation"]["bin"], "001");

    pool.close().await;
}

#[tokio::test]
async fn test_query_items() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test items
    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("SKU-001")
    .bind("Item 1")
    .bind("物品1")
    .bind("part")
    .bind("none")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("SKU-002")
    .bind("Item 2")
    .bind("物品2")
    .bind("drone")
    .bind("serial")
    .execute(pool.inner())
    .await
    .unwrap();

    let query = r#"
        query {
            items {
                edges {
                    node {
                        sku
                        name
                        nameZh
                        itemType
                        trackingMode
                    }
                }
            }
        }
    "#;

    let response = schema
        .execute(Request::new(query).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let edges = data["items"]["edges"].as_array().unwrap();
    assert!(edges.len() >= 2);
    assert!(edges.iter().any(|e| e["node"]["sku"] == "SKU-001"));
    assert!(edges.iter().any(|e| e["node"]["sku"] == "SKU-002"));

    pool.close().await;
}

#[tokio::test]
async fn test_query_item_by_sku() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test item
    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("QUERY-SKU")
    .bind("Query Item")
    .bind("查询物品")
    .bind("part")
    .bind("lot")
    .execute(pool.inner())
    .await
    .unwrap();

    let query = r#"
        query GetItem($sku: String!) {
            item(sku: $sku) {
                sku
                name
                nameZh
                trackingMode
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("sku"), Value::from("QUERY-SKU"));

    let response = schema
        .execute(Request::new(query).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["item"]["sku"], "QUERY-SKU");
    assert_eq!(data["item"]["name"], "Query Item");
    assert_eq!(data["item"]["trackingMode"], "lot");

    pool.close().await;
}

#[tokio::test]
async fn test_query_locations() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test locations
    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("WH-01")
    .bind("A")
    .bind("001")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("WH-01")
    .bind("B")
    .bind("002")
    .execute(pool.inner())
    .await
    .unwrap();

    let query = r#"
        query {
            locations {
                warehouse
                zone
                bin
            }
        }
    "#;

    let response = schema
        .execute(Request::new(query).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let locations = data["locations"].as_array().unwrap();
    assert!(locations.len() >= 2);

    pool.close().await;
}

#[tokio::test]
async fn test_receive_inventory_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user with receiving permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("receiver", "receiver@example.com", "password123")
        .await
        .unwrap();

    // Assign admin role (has all permissions including warehouse.receiving)
    let role = auth::find_role_by_name(&pool, "admin")
        .await
        .unwrap()
        .unwrap();
    auth::assign_role(&pool, user.id, role.id, None)
        .await
        .unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test item and location
    let item_id = Uuid::new_v4().to_string();
    let location_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(&item_id)
    .bind("RECV-SKU")
    .bind("Receive Item")
    .bind("接收物品")
    .bind("part")
    .bind("none")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&location_id)
    .bind("WH-01")
    .bind("RECV")
    .bind("001")
    .execute(pool.inner())
    .await
    .unwrap();

    let mutation = r#"
        mutation ReceiveInventory($sku: String!, $locationId: String!, $quantity: Int!, $reference: String) {
            receiveInventory(sku: $sku, locationId: $locationId, quantity: $quantity, reference: $reference) {
                id
                sku
                quantity
                reason
                toLocationId
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("sku"), Value::from("RECV-SKU"));
    variables.insert(Name::new("locationId"), Value::from(location_id.clone()));
    variables.insert(Name::new("quantity"), Value::from(100));
    variables.insert(Name::new("reference"), Value::from("PO-12345"));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["receiveInventory"]["sku"], "RECV-SKU");
    assert_eq!(data["receiveInventory"]["quantity"], 100);
    assert_eq!(data["receiveInventory"]["reason"], "receipt");

    // Verify inventory position was created
    let (qty_on_hand, qty_available): (i64, i64) = sqlx::query_as(
        "SELECT quantity_on_hand, quantity_available FROM inventory_positions WHERE sku = ? AND location_id = ?",
    )
    .bind("RECV-SKU")
    .bind(&location_id)
    .fetch_one(pool.inner())
    .await
    .unwrap();

    assert_eq!(qty_on_hand, 100);
    assert_eq!(qty_available, 100);

    pool.close().await;
}

#[tokio::test]
async fn test_transfer_inventory_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test item and locations
    let from_location_id = Uuid::new_v4().to_string();
    let to_location_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("XFER-SKU")
    .bind("Transfer Item")
    .bind("转移物品")
    .bind("part")
    .bind("none")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&from_location_id)
    .bind("WH-01")
    .bind("A")
    .bind("001")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&to_location_id)
    .bind("WH-01")
    .bind("B")
    .bind("002")
    .execute(pool.inner())
    .await
    .unwrap();

    // Create initial inventory position
    sqlx::query(
        r#"INSERT INTO inventory_positions
           (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
           VALUES (?, ?, ?, 'available', NULL, 100, 0, 100, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("XFER-SKU")
    .bind(&from_location_id)
    .execute(pool.inner())
    .await
    .unwrap();

    let mutation = r#"
        mutation TransferInventory($sku: String!, $fromLocationId: String!, $toLocationId: String!, $quantity: Int!, $reference: String) {
            transferInventory(sku: $sku, fromLocationId: $fromLocationId, toLocationId: $toLocationId, quantity: $quantity, reference: $reference) {
                id
                sku
                quantity
                reason
                fromLocationId
                toLocationId
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("sku"), Value::from("XFER-SKU"));
    variables.insert(Name::new("fromLocationId"), Value::from(from_location_id.clone()));
    variables.insert(Name::new("toLocationId"), Value::from(to_location_id.clone()));
    variables.insert(Name::new("quantity"), Value::from(60));
    variables.insert(Name::new("reference"), Value::from("XFER-001"));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["transferInventory"]["sku"], "XFER-SKU");
    assert_eq!(data["transferInventory"]["quantity"], 60);
    assert_eq!(data["transferInventory"]["reason"], "transfer");

    // Verify source location decreased
    let (from_qty,): (i64,) = sqlx::query_as(
        "SELECT quantity_on_hand FROM inventory_positions WHERE sku = ? AND location_id = ?",
    )
    .bind("XFER-SKU")
    .bind(&from_location_id)
    .fetch_one(pool.inner())
    .await
    .unwrap();
    assert_eq!(from_qty, 40);

    // Verify destination location increased
    let (to_qty,): (i64,) = sqlx::query_as(
        "SELECT quantity_on_hand FROM inventory_positions WHERE sku = ? AND location_id = ?",
    )
    .bind("XFER-SKU")
    .bind(&to_location_id)
    .fetch_one(pool.inner())
    .await
    .unwrap();
    assert_eq!(to_qty, 60);

    pool.close().await;
}

#[tokio::test]
async fn test_adjust_inventory_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test item and location
    let location_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("ADJ-SKU")
    .bind("Adjust Item")
    .bind("调整物品")
    .bind("part")
    .bind("none")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&location_id)
    .bind("WH-01")
    .bind("ADJ")
    .bind("001")
    .execute(pool.inner())
    .await
    .unwrap();

    // Create initial inventory position
    sqlx::query(
        r#"INSERT INTO inventory_positions
           (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
           VALUES (?, ?, ?, 'available', NULL, 100, 0, 100, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("ADJ-SKU")
    .bind(&location_id)
    .execute(pool.inner())
    .await
    .unwrap();

    let mutation = r#"
        mutation AdjustInventory($sku: String!, $locationId: String!, $quantity: Int!, $reason: String!, $reference: String) {
            adjustInventory(sku: $sku, locationId: $locationId, quantity: $quantity, reason: $reason, reference: $reference) {
                id
                sku
                quantity
                reason
                toLocationId
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("sku"), Value::from("ADJ-SKU"));
    variables.insert(Name::new("locationId"), Value::from(location_id.clone()));
    variables.insert(Name::new("quantity"), Value::from(-5)); // Decrease by 5
    variables.insert(Name::new("reason"), Value::from("cycle_count"));
    variables.insert(Name::new("reference"), Value::from("CC-001"));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["adjustInventory"]["sku"], "ADJ-SKU");
    assert_eq!(data["adjustInventory"]["quantity"], -5);
    assert_eq!(data["adjustInventory"]["reason"], "adjustment");

    // Verify inventory was adjusted
    let (qty_on_hand,): (i64,) = sqlx::query_as(
        "SELECT quantity_on_hand FROM inventory_positions WHERE sku = ? AND location_id = ?",
    )
    .bind("ADJ-SKU")
    .bind(&location_id)
    .fetch_one(pool.inner())
    .await
    .unwrap();
    assert_eq!(qty_on_hand, 95);

    pool.close().await;
}

#[tokio::test]
async fn test_query_inventory_positions() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test data
    let location_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("POS-SKU")
    .bind("Position Item")
    .bind("位置物品")
    .bind("part")
    .bind("none")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&location_id)
    .bind("WH-01")
    .bind("POS")
    .bind("001")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO inventory_positions
           (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
           VALUES (?, ?, ?, 'available', NULL, 50, 10, 40, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("POS-SKU")
    .bind(&location_id)
    .execute(pool.inner())
    .await
    .unwrap();

    let query = r#"
        query GetPositions($sku: String) {
            inventoryPositions(sku: $sku) {
                sku
                locationId
                status
                quantityOnHand
                quantityReserved
                quantityAvailable
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("sku"), Value::from("POS-SKU"));

    let response = schema
        .execute(Request::new(query).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let positions = data["inventoryPositions"].as_array().unwrap();
    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0]["sku"], "POS-SKU");
    assert_eq!(positions[0]["quantityOnHand"], 50);
    assert_eq!(positions[0]["quantityReserved"], 10);
    assert_eq!(positions[0]["quantityAvailable"], 40);

    pool.close().await;
}

#[tokio::test]
async fn test_query_inventory_movements() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test movement
    let location_id = Uuid::new_v4().to_string();
    let movement_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("MOV-SKU")
    .bind("Movement Item")
    .bind("移动物品")
    .bind("part")
    .bind("none")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&location_id)
    .bind("WH-01")
    .bind("MOV")
    .bind("001")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO inventory_movements
           (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, status_before, status_after, reference)
           VALUES (?, datetime('now'), 'testuser', 'receipt', NULL, ?, ?, 100, 'available', 'available', 'PO-001')"#,
    )
    .bind(&movement_id)
    .bind(&location_id)
    .bind("MOV-SKU")
    .execute(pool.inner())
    .await
    .unwrap();

    let query = r#"
        query GetMovements($sku: String) {
            inventoryMovements(sku: $sku) {
                id
                sku
                quantity
                reason
                actor
                toLocationId
                reference
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("sku"), Value::from("MOV-SKU"));

    let response = schema
        .execute(Request::new(query).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let movements = data["inventoryMovements"].as_array().unwrap();
    assert!(movements.len() >= 1);
    assert_eq!(movements[0]["sku"], "MOV-SKU");
    assert_eq!(movements[0]["quantity"], 100);
    assert_eq!(movements[0]["reason"], "receipt");
    assert_eq!(movements[0]["reference"], "PO-001");

    pool.close().await;
}

#[tokio::test]
async fn test_create_item_requires_permission() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user without warehouse.inventory permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("nopriv", "nopriv@example.com", "password123")
        .await
        .unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user);

    let mutation = r#"
        mutation CreateItem($input: CreateItemInput!) {
            createItem(input: $input) {
                id
                sku
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(
        Name::new("input"),
        json_to_gql_value(serde_json::json!({
            "sku": "DENIED-SKU",
            "name": "Denied Item",
            "nameZh": "拒绝物品",
            "description": "Test",
            "descriptionZh": "测试",
            "itemType": "part",
            "trackingMode": "none"
        })),
    );

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(!response.errors.is_empty());
    assert!(response.errors[0].message.contains("Permission denied"));

    pool.close().await;
}

#[tokio::test]
async fn test_transfer_inventory_insufficient_quantity() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test data with only 50 units available
    let from_location_id = Uuid::new_v4().to_string();
    let to_location_id = Uuid::new_v4().to_string();

    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("INSUF-SKU")
    .bind("Insufficient Item")
    .bind("不足物品")
    .bind("part")
    .bind("none")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&from_location_id)
    .bind("WH-01")
    .bind("A")
    .bind("001")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&to_location_id)
    .bind("WH-01")
    .bind("B")
    .bind("002")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO inventory_positions
           (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
           VALUES (?, ?, ?, 'available', NULL, 50, 0, 50, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("INSUF-SKU")
    .bind(&from_location_id)
    .execute(pool.inner())
    .await
    .unwrap();

    let mutation = r#"
        mutation TransferInventory($sku: String!, $fromLocationId: String!, $toLocationId: String!, $quantity: Int!) {
            transferInventory(sku: $sku, fromLocationId: $fromLocationId, toLocationId: $toLocationId, quantity: $quantity) {
                id
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("sku"), Value::from("INSUF-SKU"));
    variables.insert(Name::new("fromLocationId"), Value::from(from_location_id));
    variables.insert(Name::new("toLocationId"), Value::from(to_location_id));
    variables.insert(Name::new("quantity"), Value::from(100)); // Try to transfer more than available

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(!response.errors.is_empty());
    assert!(response.errors[0].message.contains("Insufficient inventory"));

    pool.close().await;
}

#[tokio::test]
async fn test_serial_genealogy_query() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test data for serial tracking
    let location_id = Uuid::new_v4().to_string();
    let serial_number = "SN-12345";

    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("SERIAL-SKU")
    .bind("Serial Item")
    .bind("序列号物品")
    .bind("drone")
    .bind("serial")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&location_id)
    .bind("WH-01")
    .bind("SER")
    .bind("001")
    .execute(pool.inner())
    .await
    .unwrap();

    // Create serial unit
    sqlx::query(
        r#"INSERT INTO serial_units (id, serial_number, sku, current_status, current_location_id, created_at)
           VALUES (?, ?, ?, 'available', ?, datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(serial_number)
    .bind("SERIAL-SKU")
    .bind(&location_id)
    .execute(pool.inner())
    .await
    .unwrap();

    // Create movement with serial number
    sqlx::query(
        r#"INSERT INTO inventory_movements
           (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, status_before, status_after, serial_numbers)
           VALUES (?, datetime('now'), 'testuser', 'receipt', NULL, ?, ?, 1, 'available', 'available', ?)"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&location_id)
    .bind("SERIAL-SKU")
    .bind(format!("[\"{}\"]", serial_number))
    .execute(pool.inner())
    .await
    .unwrap();

    let query = r#"
        query GetSerialGenealogy($serialNumber: String!) {
            serialGenealogy(serialNumber: $serialNumber) {
                movementId
                timestamp
                reason
                fromLocation
                toLocation
                sku
                statusBefore
                statusAfter
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("serialNumber"), Value::from(serial_number));

    let response = schema
        .execute(Request::new(query).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let movements = data["serialGenealogy"].as_array().unwrap();
    assert!(movements.len() >= 1);
    assert_eq!(movements[0]["reason"], "receipt");
    assert_eq!(movements[0]["sku"], "SERIAL-SKU");

    pool.close().await;
}

#[tokio::test]
async fn test_lot_genealogy_query() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test data for lot tracking
    let location_id = Uuid::new_v4().to_string();
    let lot_number = "LOT-2024-001";

    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind("LOT-SKU")
    .bind("Lot Item")
    .bind("批次物品")
    .bind("part")
    .bind("lot")
    .execute(pool.inner())
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin, created_at)
           VALUES (?, ?, ?, ?, datetime('now'))"#,
    )
    .bind(&location_id)
    .bind("WH-01")
    .bind("LOT")
    .bind("001")
    .execute(pool.inner())
    .await
    .unwrap();

    // Create lot
    let lot_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO lots (id, lot_number, sku, expiry_date, created_at)
           VALUES (?, ?, ?, '2025-12-31', datetime('now'))"#,
    )
    .bind(&lot_id)
    .bind(lot_number)
    .bind("LOT-SKU")
    .execute(pool.inner())
    .await
    .unwrap();

    // Create movement with lot_id
    sqlx::query(
        r#"INSERT INTO inventory_movements
           (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, status_before, status_after, lot_id)
           VALUES (?, datetime('now'), 'testuser', 'receipt', NULL, ?, ?, 100, 'available', 'available', ?)"#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&location_id)
    .bind("LOT-SKU")
    .bind(&lot_id)
    .execute(pool.inner())
    .await
    .unwrap();

    let query = r#"
        query GetLotGenealogy($lotNumber: String!) {
            lotGenealogy(lotNumber: $lotNumber) {
                movementId
                timestamp
                reason
                fromLocation
                toLocation
                quantity
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("lotNumber"), Value::from(lot_number));

    let response = schema
        .execute(Request::new(query).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let movements = data["lotGenealogy"].as_array().unwrap();
    assert!(movements.len() >= 1);
    assert_eq!(movements[0]["reason"], "receipt");
    assert_eq!(movements[0]["quantity"], 100);

    pool.close().await;
}

// ============================================================================
// Warehouse Receiving Tests
// ============================================================================

#[tokio::test]
async fn test_create_receiving_task_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user with warehouse.receiving permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    // Assign warehouse role
    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create receiving tasks store
    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks.clone(), pick_tasks, pack_tasks, ship_tasks);

    let mutation = r#"
        mutation CreateReceivingTask($input: CreateReceivingTaskInput!) {
            createReceivingTask(input: $input) {
                id
                taskNumber
                poId
                status
                assignedTo
            }
        }
    "#;

    let po_id = Uuid::new_v4();
    let input = serde_json::json!({
        "poId": po_id.to_string(),
        "taskNumber": "RT-001"
    });

    let mut variables = Variables::default();
    variables.insert(Name::new("input"), json_to_gql_value(input));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["createReceivingTask"]["taskNumber"], "RT-001");
    assert_eq!(data["createReceivingTask"]["status"], "PENDING");
    assert_eq!(data["createReceivingTask"]["assignedTo"], "warehouse_user");

    // Verify task was stored
    let tasks = receiving_tasks.read().unwrap();
    assert_eq!(tasks.len(), 1);

    pool.close().await;
}

#[tokio::test]
async fn test_create_receiving_task_requires_permission() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user WITHOUT warehouse.receiving permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("regular_user", "regular@example.com", "password123")
        .await
        .unwrap();

    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks, pick_tasks, pack_tasks, ship_tasks);

    let mutation = r#"
        mutation CreateReceivingTask($input: CreateReceivingTaskInput!) {
            createReceivingTask(input: $input) {
                id
                taskNumber
            }
        }
    "#;

    let po_id = Uuid::new_v4();
    let input = serde_json::json!({
        "poId": po_id.to_string(),
        "taskNumber": "RT-002"
    });

    let mut variables = Variables::default();
    variables.insert(Name::new("input"), json_to_gql_value(input));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    // Should have permission error
    assert!(!response.errors.is_empty());
    assert!(response.errors[0].message.contains("Permission denied"));

    pool.close().await;
}

#[tokio::test]
async fn test_query_receiving_task() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user with warehouse.read permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create a receiving task
    let task_id = Uuid::new_v4();
    let task = warehouse::receiving::ReceivingTask {
        id: task_id,
        task_number: "RT-003".to_string(),
        po_id: Uuid::new_v4(),
        po_number: "PO-123".to_string(),
        status: warehouse::receiving::ReceivingTaskStatus::Pending,
        lines: vec![],
        assigned_to: Some("warehouse_user".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut tasks_map = std::collections::HashMap::new();
    tasks_map.insert(task_id, task);
    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(tasks_map));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks, pick_tasks, pack_tasks, ship_tasks);

    let query = r#"
        query GetReceivingTask($id: ID!) {
            receivingTask(id: $id) {
                id
                taskNumber
                poNumber
                status
                assignedTo
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("id"), Value::from(task_id.to_string()));

    let response = schema
        .execute(Request::new(query).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["receivingTask"]["taskNumber"], "RT-003");
    assert_eq!(data["receivingTask"]["poNumber"], "PO-123");
    assert_eq!(data["receivingTask"]["status"], "PENDING");

    pool.close().await;
}

#[tokio::test]
async fn test_query_receiving_tasks_with_pagination() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create multiple receiving tasks
    let mut tasks_map = std::collections::HashMap::new();
    for i in 1..=5 {
        let task_id = Uuid::new_v4();
        let task = warehouse::receiving::ReceivingTask {
            id: task_id,
            task_number: format!("RT-{:03}", i),
            po_id: Uuid::new_v4(),
            po_number: format!("PO-{}", i),
            status: warehouse::receiving::ReceivingTaskStatus::Pending,
            lines: vec![],
            assigned_to: Some("warehouse_user".to_string()),
            created_at: chrono::Utc::now() - chrono::Duration::hours(i as i64),
            updated_at: chrono::Utc::now(),
        };
        tasks_map.insert(task_id, task);
    }

    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(tasks_map));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks, pick_tasks, pack_tasks, ship_tasks);

    let query = r#"
        query GetReceivingTasks($limit: Int, $offset: Int) {
            receivingTasks(limit: $limit, offset: $offset) {
                id
                taskNumber
                status
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("limit"), Value::from(3));
    variables.insert(Name::new("offset"), Value::from(0));

    let response = schema
        .execute(Request::new(query).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let tasks = data["receivingTasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 3); // Limited to 3

    pool.close().await;
}

#[tokio::test]
async fn test_query_receiving_tasks_by_status() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create tasks with different statuses
    let mut tasks_map = std::collections::HashMap::new();

    // Pending tasks
    for i in 1..=3 {
        let task_id = Uuid::new_v4();
        let task = warehouse::receiving::ReceivingTask {
            id: task_id,
            task_number: format!("RT-PENDING-{}", i),
            po_id: Uuid::new_v4(),
            po_number: format!("PO-{}", i),
            status: warehouse::receiving::ReceivingTaskStatus::Pending,
            lines: vec![],
            assigned_to: Some("warehouse_user".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        tasks_map.insert(task_id, task);
    }

    // Completed tasks
    for i in 1..=2 {
        let task_id = Uuid::new_v4();
        let task = warehouse::receiving::ReceivingTask {
            id: task_id,
            task_number: format!("RT-COMPLETED-{}", i),
            po_id: Uuid::new_v4(),
            po_number: format!("PO-{}", i + 10),
            status: warehouse::receiving::ReceivingTaskStatus::Completed,
            lines: vec![],
            assigned_to: Some("warehouse_user".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        tasks_map.insert(task_id, task);
    }

    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(tasks_map));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks, pick_tasks, pack_tasks, ship_tasks);

    let query = r#"
        query GetReceivingTasksByStatus($status: ReceivingTaskStatus!) {
            receivingTasksByStatus(status: $status) {
                id
                taskNumber
                status
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("status"), Value::from("PENDING"));

    let response = schema
        .execute(Request::new(query).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let tasks = data["receivingTasksByStatus"].as_array().unwrap();
    assert_eq!(tasks.len(), 3); // Only pending tasks
    assert!(tasks.iter().all(|t| t["status"] == "PENDING"));

    pool.close().await;
}

#[tokio::test]
async fn test_record_receipt_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create a receiving task with a line
    let task_id = Uuid::new_v4();
    let line_id = Uuid::new_v4();
    let line = warehouse::receiving::ReceivingLine {
        id: line_id,
        po_line_id: Uuid::new_v4(),
        sku: "TEST-SKU".to_string(),
        expected_quantity: 100,
        received_quantity: 0,
        discrepancies: vec![],
        qc_result: None,
        quarantine_status: warehouse::receiving::QuarantineStatus::None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let task = warehouse::receiving::ReceivingTask {
        id: task_id,
        task_number: "RT-004".to_string(),
        po_id: Uuid::new_v4(),
        po_number: "PO-004".to_string(),
        status: warehouse::receiving::ReceivingTaskStatus::InProgress,
        lines: vec![line],
        assigned_to: Some("warehouse_user".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut tasks_map = std::collections::HashMap::new();
    tasks_map.insert(task_id, task);
    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(tasks_map));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks.clone(), pick_tasks, pack_tasks, ship_tasks);

    let mutation = r#"
        mutation RecordReceipt($input: RecordReceiptInput!) {
            recordReceipt(input: $input) {
                id
                sku
                expectedQuantity
                receivedQuantity
            }
        }
    "#;

    let input = serde_json::json!({
        "taskId": task_id.to_string(),
        "lineId": line_id.to_string(),
        "quantity": 95
    });

    let mut variables = Variables::default();
    variables.insert(Name::new("input"), json_to_gql_value(input));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["recordReceipt"]["sku"], "TEST-SKU");
    assert_eq!(data["recordReceipt"]["expectedQuantity"], 100);
    assert_eq!(data["recordReceipt"]["receivedQuantity"], 95);

    // Verify discrepancy was created
    let tasks = receiving_tasks.read().unwrap();
    let task = tasks.get(&task_id).unwrap();
    let line = &task.lines[0];
    assert_eq!(line.discrepancies.len(), 1);
    assert_eq!(line.discrepancies[0].discrepancy_type, warehouse::receiving::DiscrepancyType::UnderReceipt);

    pool.close().await;
}

#[tokio::test]
async fn test_record_qc_outcome_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create a receiving task with a line
    let task_id = Uuid::new_v4();
    let line_id = Uuid::new_v4();
    let line = warehouse::receiving::ReceivingLine {
        id: line_id,
        po_line_id: Uuid::new_v4(),
        sku: "TEST-SKU".to_string(),
        expected_quantity: 100,
        received_quantity: 100,
        discrepancies: vec![],
        qc_result: None,
        quarantine_status: warehouse::receiving::QuarantineStatus::None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let task = warehouse::receiving::ReceivingTask {
        id: task_id,
        task_number: "RT-005".to_string(),
        po_id: Uuid::new_v4(),
        po_number: "PO-005".to_string(),
        status: warehouse::receiving::ReceivingTaskStatus::InProgress,
        lines: vec![line],
        assigned_to: Some("warehouse_user".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut tasks_map = std::collections::HashMap::new();
    tasks_map.insert(task_id, task);
    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(tasks_map));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks.clone(), pick_tasks, pack_tasks, ship_tasks);

    let mutation = r#"
        mutation RecordQCOutcome($input: RecordQCOutcomeInput!) {
            recordQcOutcome(input: $input) {
                id
                sku
                qcResult {
                    outcome
                    inspector
                    notes
                }
            }
        }
    "#;

    let input = serde_json::json!({
        "taskId": task_id.to_string(),
        "lineId": line_id.to_string(),
        "outcome": "PASS",
        "inspector": "QC-Inspector-1",
        "notes": "All items passed inspection"
    });

    let mut variables = Variables::default();
    variables.insert(Name::new("input"), json_to_gql_value(input));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["recordQcOutcome"]["sku"], "TEST-SKU");
    assert_eq!(data["recordQcOutcome"]["qcResult"]["outcome"], "PASS");
    assert_eq!(data["recordQcOutcome"]["qcResult"]["inspector"], "QC-Inspector-1");

    pool.close().await;
}

#[tokio::test]
async fn test_quarantine_line_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create a receiving task with a line that failed QC
    let task_id = Uuid::new_v4();
    let line_id = Uuid::new_v4();
    let qc_result = warehouse::receiving::QCResult {
        id: Uuid::new_v4(),
        receiving_line_id: line_id,
        outcome: warehouse::receiving::QCOutcome::Fail,
        inspector: "QC-Inspector-1".to_string(),
        notes: Some("Defects found".to_string()),
        created_at: chrono::Utc::now(),
    };

    let line = warehouse::receiving::ReceivingLine {
        id: line_id,
        po_line_id: Uuid::new_v4(),
        sku: "TEST-SKU".to_string(),
        expected_quantity: 100,
        received_quantity: 100,
        discrepancies: vec![],
        qc_result: Some(qc_result),
        quarantine_status: warehouse::receiving::QuarantineStatus::None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let task = warehouse::receiving::ReceivingTask {
        id: task_id,
        task_number: "RT-006".to_string(),
        po_id: Uuid::new_v4(),
        po_number: "PO-006".to_string(),
        status: warehouse::receiving::ReceivingTaskStatus::InProgress,
        lines: vec![line],
        assigned_to: Some("warehouse_user".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut tasks_map = std::collections::HashMap::new();
    tasks_map.insert(task_id, task);
    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(tasks_map));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks.clone(), pick_tasks, pack_tasks, ship_tasks);

    let mutation = r#"
        mutation QuarantineLine($input: QuarantineLineInput!) {
            quarantineLine(input: $input) {
                id
                sku
                quarantineStatus
            }
        }
    "#;

    let input = serde_json::json!({
        "taskId": task_id.to_string(),
        "lineId": line_id.to_string()
    });

    let mut variables = Variables::default();
    variables.insert(Name::new("input"), json_to_gql_value(input));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["quarantineLine"]["sku"], "TEST-SKU");
    assert_eq!(data["quarantineLine"]["quarantineStatus"], "QUARANTINED");

    pool.close().await;
}

#[tokio::test]
async fn test_release_from_quarantine_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create item and location for inventory update
    let item_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode)
           VALUES (?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&item_id)
    .bind("QUARANTINE-SKU")
    .bind("Quarantine Test Item")
    .bind("测试物品")
    .bind("drone")
    .bind("lot")
    .execute(pool.inner())
    .await
    .unwrap();

    let location_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin)
           VALUES (?, ?, ?, ?)"#,
    )
    .bind(&location_id)
    .bind("WH-01")
    .bind("ZONE-A")
    .bind("BIN-01")
    .execute(pool.inner())
    .await
    .unwrap();

    // Create a receiving task with a quarantined line that has passed QC
    let task_id = Uuid::new_v4();
    let line_id = Uuid::new_v4();
    let qc_result = warehouse::receiving::QCResult {
        id: Uuid::new_v4(),
        receiving_line_id: line_id,
        outcome: warehouse::receiving::QCOutcome::Pass,
        inspector: "QC-Inspector-1".to_string(),
        notes: Some("Passed after re-inspection".to_string()),
        created_at: chrono::Utc::now(),
    };

    let line = warehouse::receiving::ReceivingLine {
        id: line_id,
        po_line_id: Uuid::new_v4(),
        sku: "QUARANTINE-SKU".to_string(),
        expected_quantity: 100,
        received_quantity: 100,
        discrepancies: vec![],
        qc_result: Some(qc_result),
        quarantine_status: warehouse::receiving::QuarantineStatus::Quarantined,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let task = warehouse::receiving::ReceivingTask {
        id: task_id,
        task_number: "RT-007".to_string(),
        po_id: Uuid::new_v4(),
        po_number: "PO-007".to_string(),
        status: warehouse::receiving::ReceivingTaskStatus::InProgress,
        lines: vec![line],
        assigned_to: Some("warehouse_user".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut tasks_map = std::collections::HashMap::new();
    tasks_map.insert(task_id, task);
    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(tasks_map));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks.clone(), pick_tasks, pack_tasks, ship_tasks);

    let mutation = r#"
        mutation ReleaseFromQuarantine($input: ReleaseFromQuarantineInput!) {
            releaseFromQuarantine(input: $input) {
                id
                sku
                quarantineStatus
            }
        }
    "#;

    let input = serde_json::json!({
        "taskId": task_id.to_string(),
        "lineId": line_id.to_string(),
        "locationId": location_id
    });

    let mut variables = Variables::default();
    variables.insert(Name::new("input"), json_to_gql_value(input));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["releaseFromQuarantine"]["sku"], "QUARANTINE-SKU");
    assert_eq!(data["releaseFromQuarantine"]["quarantineStatus"], "RELEASED");

    pool.close().await;
}

#[tokio::test]
async fn test_acknowledge_discrepancy_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create a receiving task with a line that has a discrepancy
    let task_id = Uuid::new_v4();
    let line_id = Uuid::new_v4();
    let discrepancy_id = Uuid::new_v4();
    let discrepancy = warehouse::receiving::ReceivingDiscrepancy {
        id: discrepancy_id,
        receiving_line_id: line_id,
        discrepancy_type: warehouse::receiving::DiscrepancyType::UnderReceipt,
        expected_quantity: 100,
        received_quantity: 95,
        discrepancy_amount: 5,
        notes: Some("5 units missing".to_string()),
        acknowledged: false,
        created_at: chrono::Utc::now(),
    };

    let line = warehouse::receiving::ReceivingLine {
        id: line_id,
        po_line_id: Uuid::new_v4(),
        sku: "TEST-SKU".to_string(),
        expected_quantity: 100,
        received_quantity: 95,
        discrepancies: vec![discrepancy],
        qc_result: None,
        quarantine_status: warehouse::receiving::QuarantineStatus::None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let task = warehouse::receiving::ReceivingTask {
        id: task_id,
        task_number: "RT-008".to_string(),
        po_id: Uuid::new_v4(),
        po_number: "PO-008".to_string(),
        status: warehouse::receiving::ReceivingTaskStatus::InProgress,
        lines: vec![line],
        assigned_to: Some("warehouse_user".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut tasks_map = std::collections::HashMap::new();
    tasks_map.insert(task_id, task);
    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(tasks_map));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks.clone(), pick_tasks, pack_tasks, ship_tasks);

    let mutation = r#"
        mutation AcknowledgeDiscrepancy($input: AcknowledgeDiscrepancyInput!) {
            acknowledgeDiscrepancy(input: $input) {
                id
                sku
                discrepancies {
                    id
                    acknowledged
                }
            }
        }
    "#;

    let input = serde_json::json!({
        "taskId": task_id.to_string(),
        "lineId": line_id.to_string(),
        "discrepancyId": discrepancy_id.to_string()
    });

    let mut variables = Variables::default();
    variables.insert(Name::new("input"), json_to_gql_value(input));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["acknowledgeDiscrepancy"]["sku"], "TEST-SKU");
    let discrepancies = data["acknowledgeDiscrepancy"]["discrepancies"].as_array().unwrap();
    assert_eq!(discrepancies.len(), 1);
    assert_eq!(discrepancies[0]["acknowledged"], true);

    pool.close().await;
}

#[tokio::test]
async fn test_complete_receiving_task_mutation() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    // Create item and location for inventory update
    let item_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode)
           VALUES (?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&item_id)
    .bind("COMPLETE-SKU")
    .bind("Complete Test Item")
    .bind("测试物品")
    .bind("drone")
    .bind("lot")
    .execute(pool.inner())
    .await
    .unwrap();

    let location_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"INSERT INTO locations (id, warehouse, zone, bin)
           VALUES (?, ?, ?, ?)"#,
    )
    .bind(&location_id)
    .bind("WH-01")
    .bind("ZONE-A")
    .bind("BIN-01")
    .execute(pool.inner())
    .await
    .unwrap();

    // Create a receiving task ready to complete
    let task_id = Uuid::new_v4();
    let line_id = Uuid::new_v4();
    let line = warehouse::receiving::ReceivingLine {
        id: line_id,
        po_line_id: Uuid::new_v4(),
        sku: "COMPLETE-SKU".to_string(),
        expected_quantity: 100,
        received_quantity: 100,
        discrepancies: vec![],
        qc_result: Some(warehouse::receiving::QCResult {
            id: Uuid::new_v4(),
            receiving_line_id: line_id,
            outcome: warehouse::receiving::QCOutcome::Pass,
            inspector: "QC-Inspector-1".to_string(),
            notes: None,
            created_at: chrono::Utc::now(),
        }),
        quarantine_status: warehouse::receiving::QuarantineStatus::None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let task = warehouse::receiving::ReceivingTask {
        id: task_id,
        task_number: "RT-009".to_string(),
        po_id: Uuid::new_v4(),
        po_number: "PO-009".to_string(),
        status: warehouse::receiving::ReceivingTaskStatus::InProgress,
        lines: vec![line],
        assigned_to: Some("warehouse_user".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let mut tasks_map = std::collections::HashMap::new();
    tasks_map.insert(task_id, task);
    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(tasks_map));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks.clone(), pick_tasks, pack_tasks, ship_tasks);

    let mutation = r#"
        mutation CompleteReceivingTask($taskId: ID!, $locationId: String!) {
            completeReceivingTask(taskId: $taskId, locationId: $locationId) {
                id
                taskNumber
                status
            }
        }
    "#;

    let mut variables = Variables::default();
    variables.insert(Name::new("taskId"), Value::from(task_id.to_string()));
    variables.insert(Name::new("locationId"), Value::from(location_id.clone()));

    let response = schema
        .execute(Request::new(mutation).variables(variables).data(context))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["completeReceivingTask"]["taskNumber"], "RT-009");
    assert_eq!(data["completeReceivingTask"]["status"], "COMPLETED");

    pool.close().await;
}

#[tokio::test]
async fn test_receiving_workflow_state_transitions() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("warehouse_user", "warehouse@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks.clone(), pick_tasks, pack_tasks, ship_tasks);

    // Step 1: Create receiving task (should be PENDING)
    let create_mutation = r#"
        mutation CreateReceivingTask($input: CreateReceivingTaskInput!) {
            createReceivingTask(input: $input) {
                id
                status
            }
        }
    "#;

    let po_id = Uuid::new_v4();
    let input = serde_json::json!({
        "poId": po_id.to_string(),
        "taskNumber": "RT-WORKFLOW"
    });

    let mut variables = Variables::default();
    variables.insert(Name::new("input"), json_to_gql_value(input));

    let response = schema
        .execute(Request::new(create_mutation).variables(variables).data(context.clone()))
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["createReceivingTask"]["status"], "PENDING");

    // Verify task is in store
    let tasks = receiving_tasks.read().unwrap();
    assert_eq!(tasks.len(), 1);
    let task = tasks.values().next().unwrap();
    assert_eq!(task.status, warehouse::receiving::ReceivingTaskStatus::Pending);

    pool.close().await;
}

// ============================================================================
// Pagination Tests (ADV-GQL-002)
// ============================================================================

#[tokio::test]
async fn test_items_pagination_empty_results() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create admin user with permissions
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("admin", "admin@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "admin").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user.clone());

    let query = r#"
        query {
            items {
                edges {
                    node {
                        sku
                        name
                    }
                    cursor
                }
                pageInfo {
                    hasNextPage
                    hasPreviousPage
                    startCursor
                    endCursor
                }
                totalCount
            }
        }
    "#;

    let response = schema.execute(Request::new(query).data(context)).await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    assert_eq!(data["items"]["edges"].as_array().unwrap().len(), 0);
    assert_eq!(data["items"]["pageInfo"]["hasNextPage"], false);
    assert_eq!(data["items"]["pageInfo"]["hasPreviousPage"], false);
    assert!(data["items"]["pageInfo"]["startCursor"].is_null());
    assert!(data["items"]["pageInfo"]["endCursor"].is_null());

    pool.close().await;
}

#[tokio::test]
async fn test_items_pagination_first_page() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create admin user with permissions
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("admin", "admin@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "admin").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user.clone());

    // Create 5 test items
    for i in 1..=5 {
        let item_id = Uuid::new_v4().to_string();
        let sku = format!("ITEM-{:03}", i);
        sqlx::query(
            r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
               VALUES (?, ?, ?, ?, 'part', 'none', datetime('now'), datetime('now'))"#
        )
        .bind(&item_id)
        .bind(&sku)
        .bind(format!("Item {}", i))
        .bind(format!("物品 {}", i))
        .execute(pool.inner())
        .await
        .unwrap();
    }

    // Query first 3 items
    let query = r#"
        query {
            items(pagination: { first: 3 }) {
                edges {
                    node {
                        sku
                        name
                    }
                    cursor
                }
                pageInfo {
                    hasNextPage
                    hasPreviousPage
                    startCursor
                    endCursor
                }
                totalCount
            }
        }
    "#;

    let response = schema.execute(Request::new(query).data(context)).await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let edges = data["items"]["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 3);
    assert_eq!(edges[0]["node"]["sku"], "ITEM-001");
    assert_eq!(edges[1]["node"]["sku"], "ITEM-002");
    assert_eq!(edges[2]["node"]["sku"], "ITEM-003");
    assert_eq!(data["items"]["pageInfo"]["hasNextPage"], true);
    assert_eq!(data["items"]["pageInfo"]["hasPreviousPage"], false);
    assert_eq!(data["items"]["pageInfo"]["startCursor"], "ITEM-001");
    assert_eq!(data["items"]["pageInfo"]["endCursor"], "ITEM-003");
    assert_eq!(data["items"]["totalCount"], 5);

    pool.close().await;
}

#[tokio::test]
async fn test_items_pagination_cursor_navigation() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create admin user with permissions
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("admin", "admin@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "admin").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user.clone());

    // Create 10 test items
    for i in 1..=10 {
        let item_id = Uuid::new_v4().to_string();
        let sku = format!("ITEM-{:03}", i);
        sqlx::query(
            r#"INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
               VALUES (?, ?, ?, ?, 'part', 'none', datetime('now'), datetime('now'))"#
        )
        .bind(&item_id)
        .bind(&sku)
        .bind(format!("Item {}", i))
        .bind(format!("物品 {}", i))
        .execute(pool.inner())
        .await
        .unwrap();
    }

    // Query first 3 items
    let query1 = r#"
        query {
            items(pagination: { first: 3 }) {
                edges {
                    node { sku }
                    cursor
                }
                pageInfo {
                    hasNextPage
                    endCursor
                }
            }
        }
    "#;

    let response1 = schema.execute(Request::new(query1).data(context.clone())).await;
    assert!(response1.errors.is_empty());
    let data1 = response1.data.into_json().unwrap();
    let cursor = data1["items"]["pageInfo"]["endCursor"].as_str().unwrap();
    assert_eq!(cursor, "ITEM-003");

    // Query next 3 items using cursor
    let query2 = format!(
        r#"
        query {{
            items(pagination: {{ first: 3, after: "{}" }}) {{
                edges {{
                    node {{ sku }}
                    cursor
                }}
                pageInfo {{
                    hasNextPage
                    hasPreviousPage
                    startCursor
                    endCursor
                }}
            }}
        }}
    "#,
        cursor
    );

    let response2 = schema.execute(Request::new(query2).data(context)).await;
    assert!(response2.errors.is_empty());
    let data2 = response2.data.into_json().unwrap();
    let edges = data2["items"]["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 3);
    assert_eq!(edges[0]["node"]["sku"], "ITEM-004");
    assert_eq!(edges[1]["node"]["sku"], "ITEM-005");
    assert_eq!(edges[2]["node"]["sku"], "ITEM-006");
    assert_eq!(data2["items"]["pageInfo"]["hasNextPage"], true);
    assert_eq!(data2["items"]["pageInfo"]["hasPreviousPage"], true);

    pool.close().await;
}

// ============================================================================
// Error Handling Tests (ADV-GQL-002)
// ============================================================================

#[tokio::test]
async fn test_validation_error() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create admin user with permissions
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("admin", "admin@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "admin").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user.clone());

    // Try to create item with empty SKU (validation error)
    let mutation = r#"
        mutation {
            createItem(input: {
                sku: ""
                name: "Test Item"
                nameZh: "测试物品"
                itemType: "part"
                trackingMode: "none"
            }) {
                id
                sku
            }
        }
    "#;

    let response = schema.execute(Request::new(mutation).data(context)).await;

    assert!(!response.errors.is_empty());
    let error_message = response.errors[0].message.as_str();
    assert!(error_message.contains("VALIDATION_ERROR") || error_message.contains("cannot be empty"));

    pool.close().await;
}

#[tokio::test]
async fn test_authorization_error() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user WITHOUT inventory.write permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("viewer", "viewer@example.com", "password123")
        .await
        .unwrap();

    // Don't assign any role with inventory.write permission

    let context = GraphQLContext::with_user(pool.clone(), user.clone());

    // Try to create item without permission (authorization error)
    let mutation = r#"
        mutation {
            createItem(input: {
                sku: "TEST-001"
                name: "Test Item"
                nameZh: "测试物品"
                itemType: "part"
                trackingMode: "none"
            }) {
                id
                sku
            }
        }
    "#;

    let response = schema.execute(Request::new(mutation).data(context)).await;

    assert!(!response.errors.is_empty());
    let error_message = response.errors[0].message.as_str();
    assert!(error_message.contains("AUTHORIZATION_ERROR") || error_message.contains("Permission denied"));

    pool.close().await;
}

#[tokio::test]
async fn test_conflict_error() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create admin user with permissions
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("admin", "admin@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "admin").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user.clone());

    // Create first item
    let mutation1 = r#"
        mutation {
            createItem(input: {
                sku: "DUPLICATE-001"
                name: "Test Item"
                nameZh: "测试物品"
                itemType: "part"
                trackingMode: "none"
            }) {
                id
                sku
            }
        }
    "#;

    let response1 = schema.execute(Request::new(mutation1).data(context.clone())).await;
    assert!(response1.errors.is_empty());

    // Try to create duplicate item (conflict error)
    let mutation2 = r#"
        mutation {
            createItem(input: {
                sku: "DUPLICATE-001"
                name: "Another Item"
                nameZh: "另一个物品"
                itemType: "part"
                trackingMode: "none"
            }) {
                id
                sku
            }
        }
    "#;

    let response2 = schema.execute(Request::new(mutation2).data(context)).await;

    assert!(!response2.errors.is_empty());
    let error_message = response2.errors[0].message.as_str();
    assert!(error_message.contains("CONFLICT") || error_message.contains("already exists"));

    pool.close().await;
}

#[tokio::test]
async fn test_input_validation_field_errors() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create admin user with permissions
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("admin", "admin@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "admin").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let context = GraphQLContext::with_user(pool.clone(), user.clone());

    // Try to create item with invalid item_type
    let mutation = r#"
        mutation {
            createItem(input: {
                sku: "TEST-001"
                name: "Test Item"
                nameZh: "测试物品"
                itemType: "INVALID_TYPE"
                trackingMode: "none"
            }) {
                id
                sku
            }
        }
    "#;

    let response = schema.execute(Request::new(mutation).data(context)).await;

    assert!(!response.errors.is_empty());
    let error_message = response.errors[0].message.as_str();
    assert!(error_message.contains("item_type") || error_message.contains("VALIDATION"));

    pool.close().await;
}

// ============================================================================
// Bulk Mutation Tests (ADV-GQL-003)
// ============================================================================

#[tokio::test]
async fn test_bulk_adjust_inventory_all_success() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test items and locations
    let item1_id = Uuid::new_v4().to_string();
    let item2_id = Uuid::new_v4().to_string();
    let item3_id = Uuid::new_v4().to_string();
    let location_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
    )
    .bind(&item1_id).bind("TEST-001").bind("Test Item 1").bind("测试物品1").bind("drone").bind("none")
    .execute(pool.inner()).await.unwrap();

    sqlx::query(
        "INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
    )
    .bind(&item2_id).bind("TEST-002").bind("Test Item 2").bind("测试物品2").bind("battery").bind("none")
    .execute(pool.inner()).await.unwrap();

    sqlx::query(
        "INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
    )
    .bind(&item3_id).bind("TEST-003").bind("Test Item 3").bind("测试物品3").bind("part").bind("none")
    .execute(pool.inner()).await.unwrap();

    sqlx::query(
        "INSERT INTO locations (id, warehouse, zone, bin, created_at)
         VALUES (?, ?, ?, ?, datetime('now'))"
    )
    .bind(&location_id).bind("WH-01").bind("STORAGE").bind("A-01")
    .execute(pool.inner()).await.unwrap();

    // Bulk adjust inventory - all should succeed
    let mutation = r#"
        mutation BulkAdjust($inputs: [AdjustInventoryInput!]!) {
            bulkAdjustInventory(inputs: $inputs) {
                successCount
                failureCount
                results {
                    index
                    success
                    data {
                        id
                        sku
                        quantity
                        reason
                    }
                    error {
                        category
                        message
                    }
                }
            }
        }
    "#;

    let variables = serde_json::json!({
        "inputs": [
            {
                "sku": "TEST-001",
                "locationId": location_id,
                "quantity": 10,
                "reason": "Initial stock",
                "reference": "BULK-001"
            },
            {
                "sku": "TEST-002",
                "locationId": location_id,
                "quantity": 20,
                "reason": "Initial stock",
                "reference": "BULK-001"
            },
            {
                "sku": "TEST-003",
                "locationId": location_id,
                "quantity": 30,
                "reason": "Initial stock",
                "reference": "BULK-001"
            }
        ]
    });

    let response = schema
        .execute(
            Request::new(mutation)
                .variables(Variables::from_json(variables))
                .data(context),
        )
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);

    let data = response.data.into_json().unwrap();
    let bulk_result = &data["bulkAdjustInventory"];

    assert_eq!(bulk_result["successCount"], 3);
    assert_eq!(bulk_result["failureCount"], 0);
    assert_eq!(bulk_result["results"].as_array().unwrap().len(), 3);

    // Verify all results are successful
    for (i, result) in bulk_result["results"].as_array().unwrap().iter().enumerate() {
        assert_eq!(result["index"], i as i64);
        assert_eq!(result["success"], true);
        assert!(result["data"].is_object());
        assert!(result["error"].is_null());
    }

    pool.close().await;
}

#[tokio::test]
async fn test_bulk_adjust_inventory_all_fail() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Bulk adjust inventory - all should fail (validation errors)
    let mutation = r#"
        mutation BulkAdjust($inputs: [AdjustInventoryInput!]!) {
            bulkAdjustInventory(inputs: $inputs) {
                successCount
                failureCount
                results {
                    index
                    success
                    error {
                        category
                        message
                        fieldErrors {
                            field
                            message
                        }
                    }
                }
            }
        }
    "#;

    let variables = serde_json::json!({
        "inputs": [
            {
                "sku": "",
                "locationId": "loc-1",
                "quantity": 10,
                "reason": "Test",
                "reference": null
            },
            {
                "sku": "TEST-001",
                "locationId": "",
                "quantity": 20,
                "reason": "Test",
                "reference": null
            },
            {
                "sku": "TEST-002",
                "locationId": "loc-2",
                "quantity": 0,
                "reason": "Test",
                "reference": null
            }
        ]
    });

    let response = schema
        .execute(
            Request::new(mutation)
                .variables(Variables::from_json(variables))
                .data(context),
        )
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);

    let data = response.data.into_json().unwrap();
    let bulk_result = &data["bulkAdjustInventory"];

    assert_eq!(bulk_result["successCount"], 0);
    assert_eq!(bulk_result["failureCount"], 3);
    assert_eq!(bulk_result["results"].as_array().unwrap().len(), 3);

    // Verify all results are failures with validation errors
    for (i, result) in bulk_result["results"].as_array().unwrap().iter().enumerate() {
        assert_eq!(result["index"], i as i64);
        assert_eq!(result["success"], false);
        assert!(result["error"].is_object());
        assert_eq!(result["error"]["category"], "VALIDATION");
    }

    pool.close().await;
}

#[tokio::test]
async fn test_bulk_adjust_inventory_partial_failure() {
    let pool = create_test_pool().await;
    let schema = build_schema();
    let user = create_inventory_user(&pool).await;
    let context = GraphQLContext::with_user(pool.clone(), user);

    // Create test items and location
    let item1_id = Uuid::new_v4().to_string();
    let item2_id = Uuid::new_v4().to_string();
    let location_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
    )
    .bind(&item1_id).bind("VALID-001").bind("Valid Item 1").bind("有效物品1").bind("drone").bind("none")
    .execute(pool.inner()).await.unwrap();

    sqlx::query(
        "INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))"
    )
    .bind(&item2_id).bind("VALID-002").bind("Valid Item 2").bind("有效物品2").bind("battery").bind("none")
    .execute(pool.inner()).await.unwrap();

    sqlx::query(
        "INSERT INTO locations (id, warehouse, zone, bin, created_at)
         VALUES (?, ?, ?, ?, datetime('now'))"
    )
    .bind(&location_id).bind("WH-01").bind("STORAGE").bind("A-01")
    .execute(pool.inner()).await.unwrap();

    // Create initial inventory for one item
    let pos_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO inventory_positions (id, sku, location_id, status, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
         VALUES (?, ?, ?, 'available', 50, 0, 50, datetime('now'))"
    )
    .bind(&pos_id).bind("VALID-001").bind(&location_id)
    .execute(pool.inner()).await.unwrap();

    // Bulk adjust inventory - mix of success and failure
    let mutation = r#"
        mutation BulkAdjust($inputs: [AdjustInventoryInput!]!) {
            bulkAdjustInventory(inputs: $inputs) {
                successCount
                failureCount
                results {
                    index
                    success
                    data {
                        id
                        sku
                        quantity
                    }
                    error {
                        category
                        message
                    }
                }
            }
        }
    "#;

    let variables = serde_json::json!({
        "inputs": [
            {
                "sku": "VALID-001",
                "locationId": location_id,
                "quantity": 10,
                "reason": "Increase stock",
                "reference": "BULK-PARTIAL"
            },
            {
                "sku": "",
                "locationId": location_id,
                "quantity": 20,
                "reason": "Invalid SKU",
                "reference": "BULK-PARTIAL"
            },
            {
                "sku": "VALID-002",
                "locationId": location_id,
                "quantity": 30,
                "reason": "Initial stock",
                "reference": "BULK-PARTIAL"
            },
            {
                "sku": "VALID-001",
                "locationId": location_id,
                "quantity": -100,
                "reason": "Would go negative",
                "reference": "BULK-PARTIAL"
            },
            {
                "sku": "VALID-001",
                "locationId": location_id,
                "quantity": 5,
                "reason": "Another increase",
                "reference": "BULK-PARTIAL"
            }
        ]
    });

    let response = schema
        .execute(
            Request::new(mutation)
                .variables(Variables::from_json(variables))
                .data(context),
        )
        .await;

    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);

    let data = response.data.into_json().unwrap();
    let bulk_result = &data["bulkAdjustInventory"];

    assert_eq!(bulk_result["successCount"], 3);
    assert_eq!(bulk_result["failureCount"], 2);
    assert_eq!(bulk_result["results"].as_array().unwrap().len(), 5);

    let results = bulk_result["results"].as_array().unwrap();

    // Row 0: Success (VALID-001 +10)
    assert_eq!(results[0]["index"], 0);
    assert_eq!(results[0]["success"], true);
    assert_eq!(results[0]["data"]["sku"], "VALID-001");
    assert_eq!(results[0]["data"]["quantity"], 10);

    // Row 1: Failure (empty SKU - validation error)
    assert_eq!(results[1]["index"], 1);
    assert_eq!(results[1]["success"], false);
    assert_eq!(results[1]["error"]["category"], "VALIDATION");

    // Row 2: Success (VALID-002 +30)
    assert_eq!(results[2]["index"], 2);
    assert_eq!(results[2]["success"], true);
    assert_eq!(results[2]["data"]["sku"], "VALID-002");
    assert_eq!(results[2]["data"]["quantity"], 30);

    // Row 3: Failure (would go negative - validation error)
    assert_eq!(results[3]["index"], 3);
    assert_eq!(results[3]["success"], false);
    assert_eq!(results[3]["error"]["category"], "VALIDATION");
    assert!(results[3]["error"]["message"].as_str().unwrap().contains("negative"));

    // Row 4: Success (VALID-001 +5)
    assert_eq!(results[4]["index"], 4);
    assert_eq!(results[4]["success"], true);
    assert_eq!(results[4]["data"]["sku"], "VALID-001");
    assert_eq!(results[4]["data"]["quantity"], 5);

    // Verify final inventory positions
    let final_pos_1: (i64,) = sqlx::query_as(
        "SELECT quantity_on_hand FROM inventory_positions WHERE sku = ? AND location_id = ?"
    )
    .bind("VALID-001").bind(&location_id)
    .fetch_one(pool.inner()).await.unwrap();
    assert_eq!(final_pos_1.0, 65); // 50 + 10 + 5

    let final_pos_2: (i64,) = sqlx::query_as(
        "SELECT quantity_on_hand FROM inventory_positions WHERE sku = ? AND location_id = ?"
    )
    .bind("VALID-002").bind(&location_id)
    .fetch_one(pool.inner()).await.unwrap();
    assert_eq!(final_pos_2.0, 30); // 0 + 30

    pool.close().await;
}

// ============================================================================
// Shipping Tests
// ============================================================================

#[tokio::test]
async fn test_create_ship_task_and_add_line() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user with warehouse.execute permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("ship_user", "ship@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks, pick_tasks, pack_tasks, ship_tasks.clone());

    // Create ship task
    let mutation = r#"
        mutation CreateShipTask($input: CreateShipTaskInput!) {
            createShipTask(input: $input) {
                id
                taskNumber
                status
                lines {
                    id
                }
            }
        }
    "#;

    let vars_json = serde_json::json!({
        "input": {
            "taskNumber": "SHIP-001"
        }
    });
    let vars = Variables::from_json(vars_json);

    let response = schema.execute(Request::new(mutation).variables(vars).data(context.clone())).await;
    assert!(response.errors.is_empty(), "Errors: {:?}", response.errors);

    let data = response.data.into_json().unwrap();
    let task_id = data["createShipTask"]["id"].as_str().unwrap();
    assert_eq!(data["createShipTask"]["taskNumber"], "SHIP-001");
    assert_eq!(data["createShipTask"]["status"], "PENDING");
    assert_eq!(data["createShipTask"]["lines"].as_array().unwrap().len(), 0);

    // Add a line to the ship task
    let add_line_mutation = r#"
        mutation AddShipLine($input: AddShipLineInput!) {
            addShipLine(input: $input) {
                id
                lines {
                    id
                    packTaskId
                    orderId
                }
            }
        }
    "#;

    let pack_task_id = uuid::Uuid::new_v4();
    let order_id = uuid::Uuid::new_v4();
    let package_id = uuid::Uuid::new_v4();

    let add_line_vars_json = serde_json::json!({
        "input": {
            "taskId": task_id,
            "packTaskId": pack_task_id.to_string(),
            "packageIds": [package_id.to_string()],
            "orderId": order_id.to_string()
        }
    });
    let add_line_vars = Variables::from_json(add_line_vars_json);

    let add_line_response = schema.execute(Request::new(add_line_mutation).variables(add_line_vars).data(context.clone())).await;
    assert!(add_line_response.errors.is_empty(), "Errors: {:?}", add_line_response.errors);

    let add_line_data = add_line_response.data.into_json().unwrap();
    assert_eq!(add_line_data["addShipLine"]["lines"].as_array().unwrap().len(), 1);
    assert_eq!(add_line_data["addShipLine"]["lines"][0]["packTaskId"], pack_task_id.to_string());
    assert_eq!(add_line_data["addShipLine"]["lines"][0]["orderId"], order_id.to_string());

    pool.close().await;
}

#[tokio::test]
async fn test_ship_task_workflow() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user with warehouse.execute permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("ship_workflow_user", "shipflow@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks, pick_tasks, pack_tasks, ship_tasks.clone());

    // Step 1: Create ship task
    let create_mutation = r#"
        mutation CreateShipTask($input: CreateShipTaskInput!) {
            createShipTask(input: $input) {
                id
                status
            }
        }
    "#;

    let create_vars = Variables::from_json(serde_json::json!({"input": {"taskNumber": "SHIP-WORKFLOW-001"}}));
    let create_response = schema.execute(Request::new(create_mutation).variables(create_vars).data(context.clone())).await;
    assert!(create_response.errors.is_empty());
    let task_id = create_response.data.into_json().unwrap()["createShipTask"]["id"].as_str().unwrap().to_string();

    // Step 2: Add a line to the task
    let add_line_mutation = r#"
        mutation AddShipLine($input: AddShipLineInput!) {
            addShipLine(input: $input) { id }
        }
    "#;
    let pack_task_id = uuid::Uuid::new_v4();
    let order_id = uuid::Uuid::new_v4();
    let package_id = uuid::Uuid::new_v4();
    let add_line_vars = Variables::from_json(serde_json::json!({
        "input": {
            "taskId": task_id,
            "packTaskId": pack_task_id.to_string(),
            "packageIds": [package_id.to_string()],
            "orderId": order_id.to_string()
        }
    }));
    schema.execute(Request::new(add_line_mutation).variables(add_line_vars).data(context.clone())).await;

    // Step 3: Start the task
    let start_mutation = r#"
        mutation StartShipTask($taskId: ID!, $operator: String!) {
            startShipTask(taskId: $taskId, operator: $operator) {
                id
                status
                assignedTo
            }
        }
    "#;

    let start_vars = Variables::from_json(serde_json::json!({"taskId": task_id, "operator": "operator1"}));
    let start_response = schema.execute(Request::new(start_mutation).variables(start_vars).data(context.clone())).await;
    assert!(start_response.errors.is_empty(), "Start errors: {:?}", start_response.errors);
    let start_data = start_response.data.into_json().unwrap();
    assert_eq!(start_data["startShipTask"]["status"], "STAGING");
    assert_eq!(start_data["startShipTask"]["assignedTo"], "operator1");

    pool.close().await;
}

#[tokio::test]
async fn test_ship_compliance_and_dispatch() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user with warehouse.execute permission
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("ship_dispatch_user", "dispatch@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks, pick_tasks, pack_tasks, ship_tasks.clone());

    // Create and start ship task
    let create_mutation = r#"
        mutation CreateShipTask($input: CreateShipTaskInput!) {
            createShipTask(input: $input) { id }
        }
    "#;
    let create_vars = Variables::from_json(serde_json::json!({"input": {"taskNumber": "SHIP-DISPATCH-001"}}));
    let create_response = schema.execute(Request::new(create_mutation).variables(create_vars).data(context.clone())).await;
    let task_id = create_response.data.into_json().unwrap()["createShipTask"]["id"].as_str().unwrap().to_string();

    // Add a line to the task
    let add_line_mutation = r#"
        mutation AddShipLine($input: AddShipLineInput!) {
            addShipLine(input: $input) { id }
        }
    "#;
    let pack_task_id = uuid::Uuid::new_v4();
    let order_id = uuid::Uuid::new_v4();
    let package_id = uuid::Uuid::new_v4();
    let add_line_vars = Variables::from_json(serde_json::json!({
        "input": {
            "taskId": task_id,
            "packTaskId": pack_task_id.to_string(),
            "packageIds": [package_id.to_string()],
            "orderId": order_id.to_string()
        }
    }));
    schema.execute(Request::new(add_line_mutation).variables(add_line_vars).data(context.clone())).await;

    let start_mutation = r#"
        mutation StartShipTask($taskId: ID!, $operator: String!) {
            startShipTask(taskId: $taskId, operator: $operator) { id }
        }
    "#;
    let start_vars = Variables::from_json(serde_json::json!({"taskId": task_id, "operator": "operator1"}));
    schema.execute(Request::new(start_mutation).variables(start_vars).data(context.clone())).await;

    // Evaluate compliance
    let compliance_mutation = r#"
        mutation EvaluateCompliance($taskId: ID!) {
            evaluateShipCompliance(taskId: $taskId) {
                operation
                blocked
                reasons {
                    category
                    message
                }
                requiredActions
            }
        }
    "#;

    let compliance_vars = Variables::from_json(serde_json::json!({"taskId": task_id}));
    let compliance_response = schema.execute(Request::new(compliance_mutation).variables(compliance_vars).data(context.clone())).await;
    assert!(compliance_response.errors.is_empty());
    let compliance_data = compliance_response.data.into_json().unwrap();
    assert_eq!(compliance_data["evaluateShipCompliance"]["operation"], "ship_dispatch");
    assert_eq!(compliance_data["evaluateShipCompliance"]["blocked"], false);

    // Dispatch the shipment
    let dispatch_mutation = r#"
        mutation DispatchShipment($input: DispatchShipmentInput!) {
            dispatchShipment(input: $input) {
                id
                status
                carrier
                trackingNumber
                dispatchedAt
            }
        }
    "#;

    let dispatch_vars = Variables::from_json(serde_json::json!({
        "input": {
            "taskId": task_id,
            "carrier": "FedEx",
            "trackingNumber": "1234567890"
        }
    }));

    let dispatch_response = schema.execute(Request::new(dispatch_mutation).variables(dispatch_vars).data(context.clone())).await;
    assert!(dispatch_response.errors.is_empty());
    let dispatch_data = dispatch_response.data.into_json().unwrap();
    assert_eq!(dispatch_data["dispatchShipment"]["status"], "DISPATCHED");
    assert_eq!(dispatch_data["dispatchShipment"]["carrier"], "FedEx");
    assert_eq!(dispatch_data["dispatchShipment"]["trackingNumber"], "1234567890");
    assert!(dispatch_data["dispatchShipment"]["dispatchedAt"].is_string());

    // Query the ship task to verify
    let query = r#"
        query GetShipTask($id: ID!) {
            shipTask(id: $id) {
                id
                taskNumber
                status
                carrier
                trackingNumber
            }
        }
    "#;

    let query_vars = Variables::from_json(serde_json::json!({"id": task_id}));
    let query_response = schema.execute(Request::new(query).variables(query_vars).data(context.clone())).await;
    assert!(query_response.errors.is_empty());
    let query_data = query_response.data.into_json().unwrap();
    assert_eq!(query_data["shipTask"]["taskNumber"], "SHIP-DISPATCH-001");
    assert_eq!(query_data["shipTask"]["status"], "DISPATCHED");

    pool.close().await;
}

#[tokio::test]
async fn test_query_ship_tasks_by_status() {
    let pool = create_test_pool().await;
    let schema = build_schema();

    // Create user with warehouse permissions
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .create("ship_query_user", "query@example.com", "password123")
        .await
        .unwrap();

    let role = auth::find_role_by_name(&pool, "warehouse_operator").await.unwrap().unwrap();
    auth::assign_role(&pool, user.id, role.id, None).await.unwrap();

    let receiving_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pick_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let pack_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let ship_tasks = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));
    let context = GraphQLContext::with_user_and_tasks(pool.clone(), user.clone(), receiving_tasks, pick_tasks, pack_tasks, ship_tasks.clone());

    // Create multiple ship tasks with different statuses
    let create_mutation = r#"
        mutation CreateShipTask($input: CreateShipTaskInput!) {
            createShipTask(input: $input) { id }
        }
    "#;

    // Create 3 tasks
    for i in 1..=3 {
        let vars = Variables::from_json(serde_json::json!({"input": {"taskNumber": format!("SHIP-QUERY-{:03}", i)}}));
        schema.execute(Request::new(create_mutation).variables(vars).data(context.clone())).await;
    }

    // Query all ship tasks
    let query = r#"
        query GetShipTasks($limit: Int, $offset: Int) {
            shipTasks(limit: $limit, offset: $offset) {
                id
                taskNumber
                status
            }
        }
    "#;

    let query_vars = Variables::from_json(serde_json::json!({"limit": 10, "offset": 0}));
    let query_response = schema.execute(Request::new(query).variables(query_vars).data(context.clone())).await;
    assert!(query_response.errors.is_empty());
    let query_data = query_response.data.into_json().unwrap();
    assert_eq!(query_data["shipTasks"].as_array().unwrap().len(), 3);

    // Query by status (PENDING)
    let status_query = r#"
        query GetShipTasksByStatus($status: ShipTaskStatus!) {
            shipTasksByStatus(status: $status) {
                id
                taskNumber
                status
            }
        }
    "#;

    let status_vars = Variables::from_json(serde_json::json!({"status": "PENDING"}));
    let status_response = schema.execute(Request::new(status_query).variables(status_vars).data(context.clone())).await;
    assert!(status_response.errors.is_empty());
    let status_data = status_response.data.into_json().unwrap();
    assert_eq!(status_data["shipTasksByStatus"].as_array().unwrap().len(), 3);
    assert!(status_data["shipTasksByStatus"].as_array().unwrap().iter().all(|t| t["status"] == "PENDING"));

    pool.close().await;
}
