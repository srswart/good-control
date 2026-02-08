//! Packing workflow domain models and logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{WarehouseError, Result};

/// Pack task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackTaskStatus {
    /// Pack task is pending (not yet started).
    Pending,
    /// Pack task is in progress.
    InProgress,
    /// Pack task is completed.
    Completed,
    /// Pack task is cancelled.
    Cancelled,
}

/// Package (carton) entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Package ID.
    pub id: Uuid,
    /// Package number (human-readable).
    pub package_number: String,
    /// Carton type/size.
    pub carton_type: String,
    /// Package dimensions (length x width x height in cm).
    pub dimensions: Option<(f64, f64, f64)>,
    /// Package weight in kg.
    pub weight: Option<f64>,
    /// Tracking number assigned by carrier.
    pub tracking_number: Option<String>,
    /// Items packed in this package.
    pub items: Vec<PackedItem>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
}

impl Package {
    /// Create a new package.
    pub fn new(package_number: String, carton_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            package_number,
            carton_type,
            dimensions: None,
            weight: None,
            tracking_number: None,
            items: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Add an item to the package.
    pub fn add_item(&mut self, sku: String, quantity: u32, serial_numbers: Option<Vec<String>>) {
        let item = PackedItem {
            id: Uuid::new_v4(),
            sku,
            quantity,
            serial_numbers,
        };
        self.items.push(item);
    }

    /// Set package dimensions.
    pub fn set_dimensions(&mut self, length: f64, width: f64, height: f64) {
        self.dimensions = Some((length, width, height));
    }

    /// Set package weight.
    pub fn set_weight(&mut self, weight: f64) {
        self.weight = Some(weight);
    }

    /// Assign tracking number.
    pub fn assign_tracking(&mut self, tracking_number: String) {
        self.tracking_number = Some(tracking_number);
    }
}

/// Packed item record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedItem {
    /// Item ID.
    pub id: Uuid,
    /// SKU.
    pub sku: String,
    /// Quantity packed.
    pub quantity: u32,
    /// Serial numbers (if tracked).
    pub serial_numbers: Option<Vec<String>>,
}

/// Pack line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackLine {
    /// Line item ID.
    pub id: Uuid,
    /// Pick line ID this pack is for.
    pub pick_line_id: Uuid,
    /// SKU to pack.
    pub sku: String,
    /// Quantity to pack (from pick).
    pub quantity_to_pack: u32,
    /// Quantity actually packed.
    pub quantity_packed: u32,
    /// Serial numbers verified (if tracked).
    pub serial_numbers: Option<Vec<String>>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl PackLine {
    /// Create a new pack line.
    pub fn new(pick_line_id: Uuid, sku: String, quantity_to_pack: u32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            pick_line_id,
            sku,
            quantity_to_pack,
            quantity_packed: 0,
            serial_numbers: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Record packing of items.
    pub fn record_pack(&mut self, quantity: u32, serial_numbers: Option<Vec<String>>) -> Result<()> {
        if quantity == 0 {
            return Err(WarehouseError::InvalidQuantity(
                "Pack quantity must be greater than zero".to_string(),
            ));
        }

        if self.quantity_packed + quantity > self.quantity_to_pack {
            return Err(WarehouseError::InvalidQuantity(format!(
                "Cannot pack more than picked. Picked: {}, Already packed: {}, Attempting: {}",
                self.quantity_to_pack, self.quantity_packed, quantity
            )));
        }

        self.quantity_packed += quantity;
        self.serial_numbers = serial_numbers;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if line is fully packed.
    pub fn is_fully_packed(&self) -> bool {
        self.quantity_packed >= self.quantity_to_pack
    }
}

/// Pack task entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackTask {
    /// Unique pack task ID.
    pub id: Uuid,
    /// Task number (human-readable).
    pub task_number: String,
    /// Pick task ID this pack is for.
    pub pick_task_id: Uuid,
    /// Order ID.
    pub order_id: Uuid,
    /// Current status.
    pub status: PackTaskStatus,
    /// Pack lines.
    pub lines: Vec<PackLine>,
    /// Packages created for this task.
    pub packages: Vec<Package>,
    /// Assigned operator.
    pub assigned_to: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl PackTask {
    /// Create a new pack task.
    pub fn new(task_number: String, pick_task_id: Uuid, order_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_number,
            pick_task_id,
            order_id,
            status: PackTaskStatus::Pending,
            lines: Vec::new(),
            packages: Vec::new(),
            assigned_to: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a pack line to the task.
    pub fn add_line(&mut self, pick_line_id: Uuid, sku: String, quantity: u32) -> Result<()> {
        if self.status != PackTaskStatus::Pending {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only add lines to pending pack tasks".to_string(),
            ));
        }

        let line = PackLine::new(pick_line_id, sku, quantity);
        self.lines.push(line);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Start the pack task.
    pub fn start(&mut self, operator: String) -> Result<()> {
        if self.status != PackTaskStatus::Pending {
            return Err(WarehouseError::InvalidStateTransition(format!(
                "Cannot start pack task from status: {:?}",
                self.status
            )));
        }

        if self.lines.is_empty() {
            return Err(WarehouseError::InvalidStateTransition(
                "Cannot start pack task with no lines".to_string(),
            ));
        }

        self.status = PackTaskStatus::InProgress;
        self.assigned_to = Some(operator);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Create a new package for this task.
    pub fn create_package(&mut self, package_number: String, carton_type: String) -> Result<Uuid> {
        if self.status != PackTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only create packages when task is in progress".to_string(),
            ));
        }

        let package = Package::new(package_number, carton_type);
        let package_id = package.id;
        self.packages.push(package);
        self.updated_at = Utc::now();
        Ok(package_id)
    }

    /// Record packing of items into a package.
    pub fn record_pack(
        &mut self,
        line_id: Uuid,
        package_id: Uuid,
        quantity: u32,
        serial_numbers: Option<Vec<String>>,
    ) -> Result<()> {
        if self.status != PackTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only record packing when task is in progress".to_string(),
            ));
        }

        // Find and update the pack line
        let line = self
            .lines
            .iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| {
                WarehouseError::InvalidStateTransition(format!("Line {} not found", line_id))
            })?;

        line.record_pack(quantity, serial_numbers.clone())?;

        // Add item to package
        let package = self
            .packages
            .iter_mut()
            .find(|p| p.id == package_id)
            .ok_or_else(|| {
                WarehouseError::InvalidStateTransition(format!("Package {} not found", package_id))
            })?;

        package.add_item(line.sku.clone(), quantity, serial_numbers);

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set package dimensions.
    pub fn set_package_dimensions(
        &mut self,
        package_id: Uuid,
        length: f64,
        width: f64,
        height: f64,
    ) -> Result<()> {
        let package = self
            .packages
            .iter_mut()
            .find(|p| p.id == package_id)
            .ok_or_else(|| {
                WarehouseError::InvalidStateTransition(format!("Package {} not found", package_id))
            })?;

        package.set_dimensions(length, width, height);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set package weight.
    pub fn set_package_weight(&mut self, package_id: Uuid, weight: f64) -> Result<()> {
        let package = self
            .packages
            .iter_mut()
            .find(|p| p.id == package_id)
            .ok_or_else(|| {
                WarehouseError::InvalidStateTransition(format!("Package {} not found", package_id))
            })?;

        package.set_weight(weight);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Assign tracking number to package.
    pub fn assign_tracking(
        &mut self,
        package_id: Uuid,
        tracking_number: String,
    ) -> Result<()> {
        let package = self
            .packages
            .iter_mut()
            .find(|p| p.id == package_id)
            .ok_or_else(|| {
                WarehouseError::InvalidStateTransition(format!("Package {} not found", package_id))
            })?;

        package.assign_tracking(tracking_number);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Complete the pack task.
    pub fn complete(&mut self) -> Result<()> {
        if self.status != PackTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(format!(
                "Cannot complete pack task from status: {:?}",
                self.status
            )));
        }

        // Verify all lines are packed
        if !self.lines.iter().all(|l| l.is_fully_packed()) {
            return Err(WarehouseError::InvalidStateTransition(
                "Cannot complete pack task with unpacked lines".to_string(),
            ));
        }

        // Verify at least one package exists
        if self.packages.is_empty() {
            return Err(WarehouseError::InvalidStateTransition(
                "Cannot complete pack task with no packages".to_string(),
            ));
        }

        self.status = PackTaskStatus::Completed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancel the pack task.
    pub fn cancel(&mut self) -> Result<()> {
        if self.status == PackTaskStatus::Completed {
            return Err(WarehouseError::InvalidStateTransition(
                "Cannot cancel completed pack task".to_string(),
            ));
        }

        self.status = PackTaskStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if task is ready for completion.
    pub fn is_ready_for_completion(&self) -> bool {
        self.status == PackTaskStatus::InProgress
            && !self.lines.is_empty()
            && !self.packages.is_empty()
            && self.lines.iter().all(|l| l.is_fully_packed())
    }

    /// Get total lines count.
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// Get completed lines count (fully packed).
    pub fn completed_lines(&self) -> usize {
        self.lines.iter().filter(|l| l.is_fully_packed()).count()
    }

    /// Get total packages count.
    pub fn total_packages(&self) -> usize {
        self.packages.len()
    }

    /// Get total quantity to pack.
    pub fn total_quantity_to_pack(&self) -> u32 {
        self.lines.iter().map(|l| l.quantity_to_pack).sum()
    }

    /// Get total quantity packed.
    pub fn total_quantity_packed(&self) -> u32 {
        self.lines.iter().map(|l| l.quantity_packed).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_task_creation() {
        let pick_task_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
        let task = PackTask::new("PACK-001".to_string(), pick_task_id, order_id);

        assert_eq!(task.status, PackTaskStatus::Pending);
        assert_eq!(task.pick_task_id, pick_task_id);
        assert_eq!(task.order_id, order_id);
        assert!(task.lines.is_empty());
        assert!(task.packages.is_empty());
    }

    #[test]
    fn test_add_pack_line() {
        let pick_task_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
        let pick_line_id = Uuid::new_v4();
        let mut task = PackTask::new("PACK-001".to_string(), pick_task_id, order_id);

        task.add_line(pick_line_id, "SKU-001".to_string(), 10)
            .unwrap();

        assert_eq!(task.lines.len(), 1);
        assert_eq!(task.lines[0].sku, "SKU-001");
        assert_eq!(task.lines[0].quantity_to_pack, 10);
    }

    #[test]
    fn test_pack_workflow_single_package() {
        let pick_task_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
        let pick_line_id = Uuid::new_v4();
        let mut task = PackTask::new("PACK-001".to_string(), pick_task_id, order_id);

        task.add_line(pick_line_id, "SKU-001".to_string(), 10)
            .unwrap();

        // Start task
        task.start("operator1".to_string()).unwrap();
        assert_eq!(task.status, PackTaskStatus::InProgress);
        assert_eq!(task.assigned_to, Some("operator1".to_string()));

        // Create package
        let package_id = task
            .create_package("PKG-001".to_string(), "SMALL_BOX".to_string())
            .unwrap();
        assert_eq!(task.packages.len(), 1);

        // Pack items
        let line_id = task.lines[0].id;
        task.record_pack(line_id, package_id, 10, None).unwrap();
        assert_eq!(task.lines[0].quantity_packed, 10);
        assert!(task.lines[0].is_fully_packed());

        // Set package details
        task.set_package_dimensions(package_id, 30.0, 20.0, 15.0)
            .unwrap();
        task.set_package_weight(package_id, 2.5).unwrap();
        task.assign_tracking(package_id, "TRACK-123".to_string())
            .unwrap();

        assert_eq!(task.packages[0].dimensions, Some((30.0, 20.0, 15.0)));
        assert_eq!(task.packages[0].weight, Some(2.5));
        assert_eq!(
            task.packages[0].tracking_number,
            Some("TRACK-123".to_string())
        );

        // Complete task
        assert!(task.is_ready_for_completion());
        task.complete().unwrap();
        assert_eq!(task.status, PackTaskStatus::Completed);
    }

    #[test]
    fn test_pack_workflow_multi_package() {
        let pick_task_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
        let mut task = PackTask::new("PACK-001".to_string(), pick_task_id, order_id);

        task.add_line(Uuid::new_v4(), "SKU-001".to_string(), 10)
            .unwrap();
        task.add_line(Uuid::new_v4(), "SKU-002".to_string(), 5)
            .unwrap();

        task.start("operator1".to_string()).unwrap();

        // Create two packages
        let package_id_1 = task
            .create_package("PKG-001".to_string(), "SMALL_BOX".to_string())
            .unwrap();
        let package_id_2 = task
            .create_package("PKG-002".to_string(), "MEDIUM_BOX".to_string())
            .unwrap();

        // Pack first line into first package
        let line_id_1 = task.lines[0].id;
        task.record_pack(line_id_1, package_id_1, 10, None)
            .unwrap();

        // Pack second line into second package
        let line_id_2 = task.lines[1].id;
        task.record_pack(line_id_2, package_id_2, 5, None).unwrap();

        assert_eq!(task.total_packages(), 2);
        assert_eq!(task.completed_lines(), 2);
        assert_eq!(task.total_quantity_packed(), 15);

        task.complete().unwrap();
        assert_eq!(task.status, PackTaskStatus::Completed);
    }

    #[test]
    fn test_pack_with_serial_numbers() {
        let pick_task_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
        let pick_line_id = Uuid::new_v4();
        let mut task = PackTask::new("PACK-001".to_string(), pick_task_id, order_id);

        task.add_line(pick_line_id, "SKU-SERIAL-001".to_string(), 3)
            .unwrap();

        task.start("operator1".to_string()).unwrap();

        let package_id = task
            .create_package("PKG-001".to_string(), "SMALL_BOX".to_string())
            .unwrap();

        let line_id = task.lines[0].id;
        let serials = vec![
            "SN-001".to_string(),
            "SN-002".to_string(),
            "SN-003".to_string(),
        ];

        task.record_pack(line_id, package_id, 3, Some(serials.clone()))
            .unwrap();

        assert_eq!(task.lines[0].serial_numbers, Some(serials.clone()));
        assert_eq!(task.packages[0].items[0].serial_numbers, Some(serials));
    }

    #[test]
    fn test_cannot_complete_with_unpacked_lines() {
        let pick_task_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
        let mut task = PackTask::new("PACK-001".to_string(), pick_task_id, order_id);

        task.add_line(Uuid::new_v4(), "SKU-001".to_string(), 10)
            .unwrap();

        task.start("operator1".to_string()).unwrap();

        let package_id = task
            .create_package("PKG-001".to_string(), "SMALL_BOX".to_string())
            .unwrap();

        // Pack only partial quantity
        let line_id = task.lines[0].id;
        task.record_pack(line_id, package_id, 5, None).unwrap();

        assert!(!task.is_ready_for_completion());
        assert!(task.complete().is_err());
    }

    #[test]
    fn test_pack_statistics() {
        let pick_task_id = Uuid::new_v4();
        let order_id = Uuid::new_v4();
        let mut task = PackTask::new("PACK-001".to_string(), pick_task_id, order_id);

        task.add_line(Uuid::new_v4(), "SKU-001".to_string(), 10)
            .unwrap();
        task.add_line(Uuid::new_v4(), "SKU-002".to_string(), 5)
            .unwrap();

        task.start("operator1".to_string()).unwrap();

        let package_id = task
            .create_package("PKG-001".to_string(), "LARGE_BOX".to_string())
            .unwrap();

        // Pack first line fully
        let line_id_1 = task.lines[0].id;
        task.record_pack(line_id_1, package_id, 10, None).unwrap();

        // Pack second line partially
        let line_id_2 = task.lines[1].id;
        task.record_pack(line_id_2, package_id, 3, None).unwrap();

        assert_eq!(task.total_lines(), 2);
        assert_eq!(task.completed_lines(), 1);
        assert_eq!(task.total_packages(), 1);
        assert_eq!(task.total_quantity_to_pack(), 15);
        assert_eq!(task.total_quantity_packed(), 13);
    }
}


