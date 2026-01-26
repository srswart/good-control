//! Translation helpers for app shell components

use shared::i18n::SupportedLocale;

/// Translation function that takes locale explicitly
pub fn t(locale: SupportedLocale, key: &'static str) -> &'static str {
    match (locale, key) {
        // App title
        (SupportedLocale::English, "app_title") => "Controlled Goods SCM",
        (SupportedLocale::Chinese, "app_title") => "受控商品供应链管理",
        
        // Navigation
        (SupportedLocale::English, "navigation") => "Navigation",
        (SupportedLocale::Chinese, "navigation") => "导航",
        
        // Route labels
        (SupportedLocale::English, "home") => "Home",
        (SupportedLocale::Chinese, "home") => "首页",
        (SupportedLocale::English, "inventory") => "Inventory",
        (SupportedLocale::Chinese, "inventory") => "库存",
        (SupportedLocale::English, "warehouse") => "Warehouse",
        (SupportedLocale::Chinese, "warehouse") => "仓库",
        (SupportedLocale::English, "assets") => "Assets",
        (SupportedLocale::Chinese, "assets") => "资产",
        (SupportedLocale::English, "compliance") => "Compliance",
        (SupportedLocale::Chinese, "compliance") => "合规",
        (SupportedLocale::English, "orders") => "Orders",
        (SupportedLocale::Chinese, "orders") => "订单",
        (SupportedLocale::English, "logistics") => "Logistics",
        (SupportedLocale::Chinese, "logistics") => "物流",
        (SupportedLocale::English, "reporting") => "Reporting",
        (SupportedLocale::Chinese, "reporting") => "报告",
        (SupportedLocale::English, "integrations") => "Integrations",
        (SupportedLocale::Chinese, "integrations") => "集成",
        
        // Inventory UI
        (SupportedLocale::English, "items") => "Items",
        (SupportedLocale::Chinese, "items") => "物品",
        (SupportedLocale::English, "stock") => "Stock",
        (SupportedLocale::Chinese, "stock") => "库存",
        (SupportedLocale::English, "traceability") => "Traceability",
        (SupportedLocale::Chinese, "traceability") => "可追溯性",
        (SupportedLocale::English, "cycle_counts") => "Cycle Counts",
        (SupportedLocale::Chinese, "cycle_counts") => "循环盘点",
        
        // Assets UI
        (SupportedLocale::English, "drones") => "Drones",
        (SupportedLocale::Chinese, "drones") => "无人机",
        (SupportedLocale::English, "batteries") => "Batteries",
        (SupportedLocale::Chinese, "batteries") => "电池",
        
        // Compliance UI
        (SupportedLocale::English, "approvals") => "Approvals",
        (SupportedLocale::Chinese, "approvals") => "审批",
        (SupportedLocale::English, "doc_packs") => "Document Packs",
        (SupportedLocale::Chinese, "doc_packs") => "文档包",
        (SupportedLocale::English, "blocking_reasons") => "Blocking Reasons",
        (SupportedLocale::Chinese, "blocking_reasons") => "阻止原因",
        
        // Warehouse UI
        (SupportedLocale::English, "receiving") => "Receiving",
        (SupportedLocale::Chinese, "receiving") => "收货",
        (SupportedLocale::English, "putaway") => "Putaway",
        (SupportedLocale::Chinese, "putaway") => "上架",
        (SupportedLocale::English, "pick") => "Pick",
        (SupportedLocale::Chinese, "pick") => "拣货",
        (SupportedLocale::English, "pack") => "Pack",
        (SupportedLocale::Chinese, "pack") => "包装",
        (SupportedLocale::English, "ship") => "Ship",
        (SupportedLocale::Chinese, "ship") => "发货",
        
        // Logistics UI
        (SupportedLocale::English, "shipments") => "Shipments",
        (SupportedLocale::Chinese, "shipments") => "货运",
        (SupportedLocale::English, "tracking") => "Tracking",
        (SupportedLocale::Chinese, "tracking") => "追踪",
        (SupportedLocale::English, "cross_border_pack_review") => "Cross-Border Review",
        (SupportedLocale::Chinese, "cross_border_pack_review") => "跨境审查",
        
        // Procurement + Orders UI
        (SupportedLocale::English, "purchase_orders") => "Purchase Orders",
        (SupportedLocale::Chinese, "purchase_orders") => "采购订单",
        (SupportedLocale::English, "allocations") => "Allocations",
        (SupportedLocale::Chinese, "allocations") => "分配",
        (SupportedLocale::English, "fulfillment") => "Fulfillment",
        (SupportedLocale::Chinese, "fulfillment") => "履约",
        
        // Page content
        (SupportedLocale::English, "welcome") => "Welcome to Controlled Goods SCM",
        (SupportedLocale::Chinese, "welcome") => "欢迎使用受控商品供应链管理系统",
        (SupportedLocale::English, "coming_soon") => "Application content will be implemented in subsequent advances.",
        (SupportedLocale::Chinese, "coming_soon") => "应用内容将在后续推进中实现。",
        (SupportedLocale::English, "features_coming_soon") => "features coming soon.",
        (SupportedLocale::Chinese, "features_coming_soon") => "功能即将推出。",
        
        // 404 page
        (SupportedLocale::English, "not_found") => "404 - Page Not Found",
        (SupportedLocale::Chinese, "not_found") => "404 - 页面未找到",
        (SupportedLocale::English, "not_found_message") => "The page you're looking for doesn't exist.",
        (SupportedLocale::Chinese, "not_found_message") => "您要查找的页面不存在。",
        
        // Fallback
        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_english() {
        assert_eq!(t(SupportedLocale::English, "app_title"), "Controlled Goods SCM");
        assert_eq!(t(SupportedLocale::English, "navigation"), "Navigation");
        assert_eq!(t(SupportedLocale::English, "home"), "Home");
        assert_eq!(t(SupportedLocale::English, "welcome"), "Welcome to Controlled Goods SCM");
    }

    #[test]
    fn test_translation_chinese() {
        assert_eq!(t(SupportedLocale::Chinese, "app_title"), "受控商品供应链管理");
        assert_eq!(t(SupportedLocale::Chinese, "navigation"), "导航");
        assert_eq!(t(SupportedLocale::Chinese, "home"), "首页");
        assert_eq!(t(SupportedLocale::Chinese, "welcome"), "欢迎使用受控商品供应链管理系统");
    }

    #[test]
    fn test_translation_fallback() {
        // Unknown key should return the key itself
        assert_eq!(t(SupportedLocale::English, "unknown_key"), "unknown_key");
        assert_eq!(t(SupportedLocale::Chinese, "unknown_key"), "unknown_key");
    }

    #[test]
    fn test_all_route_labels_translated() {
        let routes = vec!["home", "inventory", "warehouse", "assets", "compliance", "orders", "logistics", "reporting", "integrations"];
        
        for route in routes {
            // English translation should not be empty
            let en = t(SupportedLocale::English, route);
            assert!(!en.is_empty(), "English translation missing for {}", route);
            assert_ne!(en, route, "English translation should not be the key for {}", route);
            
            // Chinese translation should not be empty
            let zh = t(SupportedLocale::Chinese, route);
            assert!(!zh.is_empty(), "Chinese translation missing for {}", route);
            assert_ne!(zh, route, "Chinese translation should not be the key for {}", route);
        }
    }
}
