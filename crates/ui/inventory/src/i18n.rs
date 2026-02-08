//! Translation helpers for inventory UI components

use shared::i18n::SupportedLocale;

/// Translation function for inventory UI
pub fn t(locale: SupportedLocale, key: &'static str) -> &'static str {
    match (locale, key) {
        // Items list
        (SupportedLocale::English, "items") => "Items",
        (SupportedLocale::Chinese, "items") => "物品",
        (SupportedLocale::English, "items_total") => "total",
        (SupportedLocale::Chinese, "items_total") => "总计",
        (SupportedLocale::English, "search_placeholder") => "Search by SKU or name...",
        (SupportedLocale::Chinese, "search_placeholder") => "按SKU或名称搜索...",
        (SupportedLocale::English, "search") => "Search",
        (SupportedLocale::Chinese, "search") => "搜索",
        (SupportedLocale::English, "loading") => "Loading...",
        (SupportedLocale::Chinese, "loading") => "加载中...",
        (SupportedLocale::English, "error_loading_items") => "Error loading items",
        (SupportedLocale::Chinese, "error_loading_items") => "加载物品时出错",
        (SupportedLocale::English, "retry") => "Retry",
        (SupportedLocale::Chinese, "retry") => "重试",
        (SupportedLocale::English, "no_items_found") => "No items found",
        (SupportedLocale::Chinese, "no_items_found") => "未找到物品",
        (SupportedLocale::English, "no_items_message") => "Try adjusting your search or add new items",
        (SupportedLocale::Chinese, "no_items_message") => "尝试调整搜索或添加新物品",
        (SupportedLocale::English, "showing") => "Showing",
        (SupportedLocale::Chinese, "showing") => "显示",
        (SupportedLocale::English, "of") => "of",
        (SupportedLocale::Chinese, "of") => "共",
        (SupportedLocale::English, "items_count") => "items",
        (SupportedLocale::Chinese, "items_count") => "项",
        (SupportedLocale::English, "previous") => "Previous",
        (SupportedLocale::Chinese, "previous") => "上一页",
        (SupportedLocale::English, "next") => "Next",
        (SupportedLocale::Chinese, "next") => "下一页",
        
        // Table headers
        (SupportedLocale::English, "sku") => "SKU",
        (SupportedLocale::Chinese, "sku") => "SKU",
        (SupportedLocale::English, "name_en") => "Name (EN)",
        (SupportedLocale::Chinese, "name_en") => "名称（英文）",
        (SupportedLocale::English, "name_zh") => "Name (ZH)",
        (SupportedLocale::Chinese, "name_zh") => "名称（中文）",
        (SupportedLocale::English, "type") => "Type",
        (SupportedLocale::Chinese, "type") => "类型",
        (SupportedLocale::English, "tracking") => "Tracking",
        (SupportedLocale::Chinese, "tracking") => "追踪",
        
        // Error categories
        (SupportedLocale::English, "validation_error") => "Validation Error",
        (SupportedLocale::Chinese, "validation_error") => "验证错误",
        (SupportedLocale::English, "authorization_error") => "Authorization Error",
        (SupportedLocale::Chinese, "authorization_error") => "授权错误",
        (SupportedLocale::English, "compliance_block") => "Compliance Block",
        (SupportedLocale::Chinese, "compliance_block") => "合规阻止",
        (SupportedLocale::English, "not_found") => "Not Found",
        (SupportedLocale::Chinese, "not_found") => "未找到",
        (SupportedLocale::English, "conflict") => "Conflict",
        (SupportedLocale::Chinese, "conflict") => "冲突",
        (SupportedLocale::English, "internal_error") => "Internal Error",
        (SupportedLocale::Chinese, "internal_error") => "内部错误",
        (SupportedLocale::English, "network_error") => "Network Error",
        (SupportedLocale::Chinese, "network_error") => "网络错误",
        (SupportedLocale::English, "http_error") => "HTTP Error",
        (SupportedLocale::Chinese, "http_error") => "HTTP错误",
        (SupportedLocale::English, "graphql_error") => "GraphQL Error",
        (SupportedLocale::Chinese, "graphql_error") => "GraphQL错误",
        (SupportedLocale::English, "parse_error") => "Parse Error",
        (SupportedLocale::Chinese, "parse_error") => "解析错误",
        (SupportedLocale::English, "invalid_response") => "Invalid Response",
        (SupportedLocale::Chinese, "invalid_response") => "无效响应",
        (SupportedLocale::English, "fields") => "Fields",
        (SupportedLocale::Chinese, "fields") => "字段",
        
        // Fallback
        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_english() {
        assert_eq!(t(SupportedLocale::English, "items"), "Items");
        assert_eq!(t(SupportedLocale::English, "search"), "Search");
        assert_eq!(t(SupportedLocale::English, "previous"), "Previous");
        assert_eq!(t(SupportedLocale::English, "next"), "Next");
    }

    #[test]
    fn test_translation_chinese() {
        assert_eq!(t(SupportedLocale::Chinese, "items"), "物品");
        assert_eq!(t(SupportedLocale::Chinese, "search"), "搜索");
        assert_eq!(t(SupportedLocale::Chinese, "previous"), "上一页");
        assert_eq!(t(SupportedLocale::Chinese, "next"), "下一页");
    }
}

