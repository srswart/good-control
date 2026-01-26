//! Temporary component showcase page for reviewing design system components
//! This will be removed once proper routing and app shell are implemented.

use dioxus::prelude::*;
use shared::i18n::SupportedLocale;
use ui_design_system::{
    Button, ButtonVariant, TextInput, Textarea, ValidationError, Modal, Toast, ToastLevel, Table, TableColumn,
};

/// Simple translation function for demo purposes
/// This will be replaced with the proper i18n system in ADV-I18N-001
fn t(locale: SupportedLocale, key: &'static str) -> &'static str {
    match (locale, key) {
        // Page title and description
        (SupportedLocale::English, "page_title") => "Design System Component Showcase",
        (SupportedLocale::Chinese, "page_title") => "设计系统组件展示",
        (SupportedLocale::English, "page_description") => "Temporary demo page for reviewing design system components",
        (SupportedLocale::Chinese, "page_description") => "用于查看设计系统组件的临时演示页面",
        
        // Section headers
        (SupportedLocale::English, "buttons") => "Buttons",
        (SupportedLocale::Chinese, "buttons") => "按钮",
        (SupportedLocale::English, "form_inputs") => "Form Inputs",
        (SupportedLocale::Chinese, "form_inputs") => "表单输入",
        (SupportedLocale::English, "validation_errors") => "Validation Errors",
        (SupportedLocale::Chinese, "validation_errors") => "验证错误",
        (SupportedLocale::English, "modal") => "Modal",
        (SupportedLocale::Chinese, "modal") => "模态框",
        (SupportedLocale::English, "toasts") => "Toasts",
        (SupportedLocale::Chinese, "toasts") => "通知",
        (SupportedLocale::English, "table") => "Table",
        (SupportedLocale::Chinese, "table") => "表格",
        
        // Button labels
        (SupportedLocale::English, "primary") => "Primary",
        (SupportedLocale::Chinese, "primary") => "主要",
        (SupportedLocale::English, "secondary") => "Secondary",
        (SupportedLocale::Chinese, "secondary") => "次要",
        (SupportedLocale::English, "danger") => "Danger",
        (SupportedLocale::Chinese, "danger") => "危险",
        (SupportedLocale::English, "disabled") => "Disabled",
        (SupportedLocale::Chinese, "disabled") => "禁用",
        (SupportedLocale::English, "primary_button") => "Primary Button",
        (SupportedLocale::Chinese, "primary_button") => "主要按钮",
        (SupportedLocale::English, "secondary_button") => "Secondary Button",
        (SupportedLocale::Chinese, "secondary_button") => "次要按钮",
        (SupportedLocale::English, "delete") => "Delete",
        (SupportedLocale::Chinese, "delete") => "删除",
        (SupportedLocale::English, "disabled_button") => "Disabled Button",
        (SupportedLocale::Chinese, "disabled_button") => "禁用按钮",
        
        // Form labels
        (SupportedLocale::English, "text_input") => "Text Input",
        (SupportedLocale::Chinese, "text_input") => "文本输入",
        (SupportedLocale::English, "textarea") => "Textarea",
        (SupportedLocale::Chinese, "textarea") => "多行文本",
        (SupportedLocale::English, "disabled_input") => "Disabled Input",
        (SupportedLocale::Chinese, "disabled_input") => "禁用输入",
        (SupportedLocale::English, "enter_text") => "Enter text here...",
        (SupportedLocale::Chinese, "enter_text") => "在此输入文本...",
        (SupportedLocale::English, "enter_multiline") => "Enter multi-line text...",
        (SupportedLocale::Chinese, "enter_multiline") => "输入多行文本...",
        (SupportedLocale::English, "value") => "Value:",
        (SupportedLocale::Chinese, "value") => "值：",
        
        // Validation messages
        (SupportedLocale::English, "field_required") => "This field is required",
        (SupportedLocale::Chinese, "field_required") => "此字段为必填项",
        (SupportedLocale::English, "email_invalid") => "Email format is invalid",
        (SupportedLocale::Chinese, "email_invalid") => "电子邮件格式无效",
        (SupportedLocale::English, "password_length") => "Password must be at least 8 characters",
        (SupportedLocale::Chinese, "password_length") => "密码必须至少8个字符",
        
        // Modal
        (SupportedLocale::English, "open_modal") => "Open Modal",
        (SupportedLocale::Chinese, "open_modal") => "打开模态框",
        (SupportedLocale::English, "example_modal") => "Example Modal",
        (SupportedLocale::Chinese, "example_modal") => "示例模态框",
        (SupportedLocale::English, "modal_content") => "This is a modal dialog. You can close it by clicking the backdrop, the × button, or the close button below.",
        (SupportedLocale::Chinese, "modal_content") => "这是一个模态对话框。您可以通过点击背景、×按钮或下面的关闭按钮来关闭它。",
        (SupportedLocale::English, "cancel") => "Cancel",
        (SupportedLocale::Chinese, "cancel") => "取消",
        (SupportedLocale::English, "confirm") => "Confirm",
        (SupportedLocale::Chinese, "confirm") => "确认",
        
        // Toasts
        (SupportedLocale::English, "show_info_toast") => "Show Info Toast",
        (SupportedLocale::Chinese, "show_info_toast") => "显示信息通知",
        (SupportedLocale::English, "show_success_toast") => "Show Success Toast",
        (SupportedLocale::Chinese, "show_success_toast") => "显示成功通知",
        (SupportedLocale::English, "show_warning_toast") => "Show Warning Toast",
        (SupportedLocale::Chinese, "show_warning_toast") => "显示警告通知",
        (SupportedLocale::English, "show_error_toast") => "Show Error Toast",
        (SupportedLocale::Chinese, "show_error_toast") => "显示错误通知",
        
        // Table headers
        (SupportedLocale::English, "name") => "Name",
        (SupportedLocale::Chinese, "name") => "名称",
        (SupportedLocale::English, "status") => "Status",
        (SupportedLocale::Chinese, "status") => "状态",
        (SupportedLocale::English, "location") => "Location",
        (SupportedLocale::Chinese, "location") => "位置",
        (SupportedLocale::English, "quantity") => "Quantity",
        (SupportedLocale::Chinese, "quantity") => "数量",
        
        // Footer
        (SupportedLocale::English, "footer_note") => "This is a temporary showcase page. Remove once routing is implemented.",
        (SupportedLocale::Chinese, "footer_note") => "这是一个临时展示页面。实现路由后应删除。",
        
        // Language switcher
        (SupportedLocale::English, "language") => "Language",
        (SupportedLocale::Chinese, "language") => "语言",
        
        // Default fallback
        _ => key,
    }
}

#[component]
pub fn ComponentShowcase(
    on_back: EventHandler,
) -> Element {
    let mut locale = use_signal(|| SupportedLocale::English);
    let mut modal_open = use_signal(|| false);
    let mut toast_visible = use_signal(|| false);
    let mut toast_level = use_signal(|| ToastLevel::Info);
    let mut toast_message = use_signal(|| "This is a toast notification".to_string());
    let mut text_input_value = use_signal(|| String::new());
    let mut textarea_value = use_signal(|| String::new());

    rsx! {
        div {
            class: "min-h-screen bg-gray-50 flex flex-col",
            // Header with Back button and Language Switcher
            header {
                class: "bg-white shadow-sm border-b border-gray-200",
                div {
                    class: "max-w-6xl mx-auto px-8 py-4",
                    div {
                        class: "flex justify-between items-center mb-4",
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| on_back.call(()),
                            "← Back to App"
                        }
                        // Language Switcher
                        div {
                            class: "flex items-center gap-2",
                            span {
                                class: "text-sm text-gray-600",
                                {t(locale(), "language")}
                                ": "
                            }
                            div {
                                class: "flex gap-1 border border-gray-300 rounded-md overflow-hidden",
                                for supported_locale in SupportedLocale::all() {
                                    Button {
                                        variant: if locale() == *supported_locale {
                                            ButtonVariant::Primary
                                        } else {
                                            ButtonVariant::Secondary
                                        },
                                        class: Some("rounded-none border-0".to_string()),
                                        onclick: move |_| locale.set(*supported_locale),
                                        {supported_locale.native_name()}
                                    }
                                }
                            }
                        }
                    }
                    div {
                        h1 {
                            class: "text-3xl font-bold text-gray-900 mb-2",
                            {t(locale(), "page_title")}
                        }
                        p {
                            class: "text-gray-600",
                            {t(locale(), "page_description")}
                        }
                    }
                }
            }
            
            // Main content
            main {
                class: "flex-1 max-w-6xl mx-auto w-full p-8",

                // Buttons Section
                section {
                    class: "mb-12 bg-white rounded-lg shadow p-6",
                    h2 {
                        class: "text-2xl font-semibold text-gray-800 mb-4",
                        {t(locale(), "buttons")}
                    }
                    div {
                        class: "space-y-4",
                        div {
                            class: "flex gap-4 items-center",
                            span {
                                class: "w-32 text-sm text-gray-600",
                                {t(locale(), "primary")}
                                ": "
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| {
                                    toast_message.set(if locale() == SupportedLocale::English {
                                        "Primary button clicked!".to_string()
                                    } else {
                                        "主要按钮已点击！".to_string()
                                    });
                                    toast_level.set(ToastLevel::Success);
                                    toast_visible.set(true);
                                },
                                {t(locale(), "primary_button")}
                            }
                        }
                        div {
                            class: "flex gap-4 items-center",
                            span {
                                class: "w-32 text-sm text-gray-600",
                                {t(locale(), "secondary")}
                                ": "
                            }
                            Button {
                                variant: ButtonVariant::Secondary,
                                onclick: move |_| {
                                    toast_message.set(if locale() == SupportedLocale::English {
                                        "Secondary button clicked!".to_string()
                                    } else {
                                        "次要按钮已点击！".to_string()
                                    });
                                    toast_level.set(ToastLevel::Info);
                                    toast_visible.set(true);
                                },
                                {t(locale(), "secondary_button")}
                            }
                        }
                        div {
                            class: "flex gap-4 items-center",
                            span {
                                class: "w-32 text-sm text-gray-600",
                                {t(locale(), "danger")}
                                ": "
                            }
                            Button {
                                variant: ButtonVariant::Danger,
                                onclick: move |_| {
                                    toast_message.set(if locale() == SupportedLocale::English {
                                        "Danger button clicked!".to_string()
                                    } else {
                                        "危险按钮已点击！".to_string()
                                    });
                                    toast_level.set(ToastLevel::Error);
                                    toast_visible.set(true);
                                },
                                {t(locale(), "delete")}
                            }
                        }
                        div {
                            class: "flex gap-4 items-center",
                            span {
                                class: "w-32 text-sm text-gray-600",
                                {t(locale(), "disabled")}
                                ": "
                            }
                            Button {
                                disabled: true,
                                {t(locale(), "disabled_button")}
                            }
                        }
                    }
                }

                // Form Inputs Section
                section {
                    class: "mb-12 bg-white rounded-lg shadow p-6",
                    h2 {
                        class: "text-2xl font-semibold text-gray-800 mb-4",
                        {t(locale(), "form_inputs")}
                    }
                    div {
                        class: "space-y-6",
                        div {
                            class: "space-y-2",
                            label {
                                class: "block text-sm font-medium text-gray-700",
                                {t(locale(), "text_input")}
                            }
                            TextInput {
                                value: text_input_value(),
                                placeholder: Some(t(locale(), "enter_text").to_string()),
                                oninput: move |e: FormEvent| {
                                    text_input_value.set(e.value());
                                },
                            }
                            if !text_input_value().is_empty() {
                                p {
                                    class: "text-sm text-gray-500",
                                    {t(locale(), "value")} " " {text_input_value()}
                                }
                            }
                        }
                        div {
                            class: "space-y-2",
                            label {
                                class: "block text-sm font-medium text-gray-700",
                                {t(locale(), "textarea")}
                            }
                            Textarea {
                                value: textarea_value(),
                                placeholder: Some(t(locale(), "enter_multiline").to_string()),
                                rows: 4,
                                oninput: move |e: FormEvent| {
                                    textarea_value.set(e.value());
                                },
                            }
                            if !textarea_value().is_empty() {
                                p {
                                    class: "text-sm text-gray-500",
                                    {t(locale(), "value")} " " {textarea_value()}
                                }
                            }
                        }
                        div {
                            class: "space-y-2",
                            label {
                                class: "block text-sm font-medium text-gray-700",
                                {t(locale(), "disabled_input")}
                            }
                            TextInput {
                                value: if locale() == SupportedLocale::English {
                                    "Disabled value".to_string()
                                } else {
                                    "禁用值".to_string()
                                },
                                disabled: true,
                                oninput: |_| {},
                            }
                        }
                    }
                }

                // Validation Errors Section
                section {
                    class: "mb-12 bg-white rounded-lg shadow p-6",
                    h2 {
                        class: "text-2xl font-semibold text-gray-800 mb-4",
                        {t(locale(), "validation_errors")}
                    }
                    div {
                        class: "space-y-4",
                        ValidationError {
                            message: t(locale(), "field_required").to_string()
                        }
                        ValidationError {
                            message: t(locale(), "email_invalid").to_string()
                        }
                        ValidationError {
                            message: t(locale(), "password_length").to_string()
                        }
                    }
                }

                // Modal Section
                section {
                    class: "mb-12 bg-white rounded-lg shadow p-6",
                    h2 {
                        class: "text-2xl font-semibold text-gray-800 mb-4",
                        {t(locale(), "modal")}
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| modal_open.set(true),
                        {t(locale(), "open_modal")}
                    }
                    Modal {
                        open: modal_open(),
                        title: t(locale(), "example_modal").to_string(),
                        onclose: move || modal_open.set(false),
                        div {
                            class: "space-y-4",
                            p {
                                class: "text-gray-700",
                                {t(locale(), "modal_content")}
                            }
                            div {
                                class: "flex justify-end gap-2",
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    onclick: move |_| modal_open.set(false),
                                    {t(locale(), "cancel")}
                                }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| {
                                        modal_open.set(false);
                                        toast_message.set(if locale() == SupportedLocale::English {
                                            "Action confirmed!".to_string()
                                        } else {
                                            "操作已确认！".to_string()
                                        });
                                        toast_level.set(ToastLevel::Success);
                                        toast_visible.set(true);
                                    },
                                    {t(locale(), "confirm")}
                                }
                            }
                        }
                    }
                }

                // Toast Section
                section {
                    class: "mb-12 bg-white rounded-lg shadow p-6",
                    h2 {
                        class: "text-2xl font-semibold text-gray-800 mb-4",
                        {t(locale(), "toasts")}
                    }
                    div {
                        class: "grid grid-cols-2 gap-4",
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                toast_message.set(if locale() == SupportedLocale::English {
                                    "Info toast notification".to_string()
                                } else {
                                    "信息通知".to_string()
                                });
                                toast_level.set(ToastLevel::Info);
                                toast_visible.set(true);
                            },
                            {t(locale(), "show_info_toast")}
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                toast_message.set(if locale() == SupportedLocale::English {
                                    "Success! Operation completed.".to_string()
                                } else {
                                    "成功！操作已完成。".to_string()
                                });
                                toast_level.set(ToastLevel::Success);
                                toast_visible.set(true);
                            },
                            {t(locale(), "show_success_toast")}
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                toast_message.set(if locale() == SupportedLocale::English {
                                    "Warning: Please review this action".to_string()
                                } else {
                                    "警告：请审查此操作".to_string()
                                });
                                toast_level.set(ToastLevel::Warning);
                                toast_visible.set(true);
                            },
                            {t(locale(), "show_warning_toast")}
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                toast_message.set(if locale() == SupportedLocale::English {
                                    "Error: Something went wrong".to_string()
                                } else {
                                    "错误：出现了问题".to_string()
                                });
                                toast_level.set(ToastLevel::Error);
                                toast_visible.set(true);
                            },
                            {t(locale(), "show_error_toast")}
                        }
                    }
                    Toast {
                        visible: toast_visible(),
                        message: toast_message(),
                        level: toast_level(),
                        onclose: move || toast_visible.set(false),
                    }
                }

                // Table Section
                section {
                    class: "mb-12 bg-white rounded-lg shadow p-6",
                    h2 {
                        class: "text-2xl font-semibold text-gray-800 mb-4",
                        {t(locale(), "table")}
                    }
                    Table {
                        columns: vec![
                            TableColumn { header: t(locale(), "name").to_string(), key: "name".to_string() },
                            TableColumn { header: t(locale(), "status").to_string(), key: "status".to_string() },
                            TableColumn { header: t(locale(), "location").to_string(), key: "location".to_string() },
                            TableColumn { header: t(locale(), "quantity").to_string(), key: "quantity".to_string() },
                        ],
                        rows: vec![
                            vec![
                                if locale() == SupportedLocale::English { "Drone Battery A123" } else { "无人机电池 A123" }.to_string(),
                                if locale() == SupportedLocale::English { "Active" } else { "活跃" }.to_string(),
                                if locale() == SupportedLocale::English { "Warehouse A" } else { "仓库 A" }.to_string(),
                                "15".to_string()
                            ],
                            vec![
                                if locale() == SupportedLocale::English { "Propeller Set P4" } else { "螺旋桨组 P4" }.to_string(),
                                if locale() == SupportedLocale::English { "In Stock" } else { "有库存" }.to_string(),
                                if locale() == SupportedLocale::English { "Warehouse B" } else { "仓库 B" }.to_string(),
                                "42".to_string()
                            ],
                            vec![
                                if locale() == SupportedLocale::English { "GPS Module G7" } else { "GPS 模块 G7" }.to_string(),
                                if locale() == SupportedLocale::English { "Low Stock" } else { "库存不足" }.to_string(),
                                if locale() == SupportedLocale::English { "Warehouse A" } else { "仓库 A" }.to_string(),
                                "3".to_string()
                            ],
                            vec![
                                if locale() == SupportedLocale::English { "Camera C2" } else { "相机 C2" }.to_string(),
                                if locale() == SupportedLocale::English { "Active" } else { "活跃" }.to_string(),
                                if locale() == SupportedLocale::English { "Warehouse C" } else { "仓库 C" }.to_string(),
                                "28".to_string()
                            ],
                        ],
                        striped: true,
                    }
                }

            }
            
            // Footer
            footer {
                class: "bg-white border-t border-gray-200 mt-auto",
                div {
                    class: "max-w-6xl mx-auto px-8 py-4",
                    div {
                        class: "flex justify-between items-center",
                        p {
                            class: "text-sm text-gray-500",
                            {t(locale(), "footer_note")}
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| on_back.call(()),
                            "← Back to App"
                        }
                    }
                }
            }
        }
    }
}
