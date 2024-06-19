use super::component::UIComponent;

pub mod new_instance;
pub mod confirmation;
pub mod list_select;
pub mod rename_instance;
pub mod modify_instance_executable;

/// Return true if the information is valid:
/// - Name is non-empty and alphanumerics
pub fn is_instance_name_valid(name: &str) -> bool {
    let name = name.trim();
    !name.is_empty() && name.chars().all(|c| char::is_alphanumeric(c) || matches!(c, '_' | '-' | ' '))
}

/// Trait that reprersents a UIComponent which can be consumed and transformed into a message
pub trait ConsumablePopup<T>: UIComponent<T> {
    /// Transform Self into a message of type T
    /// self will be dropped immediately after this function is called
    /// A non-referenced Self cannot be passed due to the nature of dyn Traits
    /// in that they cannot be sized
    fn consume(&mut self) -> Option<T>;
}

/// Type alias to make boxing the trait easier
pub type BoxedConsumablePopup<T> = Box<dyn ConsumablePopup<T>>;
