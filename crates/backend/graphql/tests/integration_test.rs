//! Integration tests for GraphQL schema.

use async_graphql::{Request, Variables};
use auth::{SqliteUserRepository, UserRepository};
use graphql::{build_schema, GraphQLContext};
use persistence::{DbPool, DbPoolConfig};
use uuid::Uuid;

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
