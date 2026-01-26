//! @arrive component: design-system
//! Design system: reusable components, forms, validation patterns.
//!
//! This crate provides reusable UI components for the frontend:
//! - Buttons (primary, secondary, danger variants)
//! - Form inputs (text, textarea, validation)
//! - Modals (dialogs, confirmations)
//! - Toasts (notifications, feedback)
//! - Tables (data display)
//! - Validation (error display patterns)

pub mod button;
pub mod form;
pub mod modal;
pub mod toast;
pub mod table;
pub mod validation;

pub use button::{Button, ButtonVariant};
pub use form::{TextInput, Textarea};
pub use modal::Modal;
pub use toast::{Toast, ToastLevel};
pub use table::{Table, TableColumn};
pub use validation::ValidationError;
