//! Inventory management CLI for ops/admin workflows.
//!
//! This CLI provides commands for:
//! - Database migrations and backups
//! - Bulk import/export
//! - Compliance validation and document pack generation
//! - Kingdee sync operations
//! - Diagnostics and health checks

use anyhow::Result;
use clap::{Parser, Subcommand};
use cli_foundation::{init_logging, CliConfig, OutputFormat, OutputWriter};
use std::path::PathBuf;

/// Inventory management CLI
#[derive(Parser)]
#[command(name = "inventory-cli")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, global = true)]
    log_level: Option<String>,

    /// Output format (human, json)
    #[arg(short = 'f', long = "format", global = true, value_name = "FORMAT")]
    format: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Database operations (migrate, backup, restore, verify)
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },

    /// Import data (items, locations, suppliers, customers, initial stock)
    Import {
        #[command(subcommand)]
        command: ImportCommands,
    },

    /// Export data (inventory snapshots, audit extracts, shipment manifests)
    Export {
        #[command(subcommand)]
        command: ExportCommands,
    },

    /// Compliance operations (validate gates, list missing docs, build packs)
    Compliance {
        #[command(subcommand)]
        command: ComplianceCommands,
    },

    /// Kingdee sync operations (sync, retry, reconcile, mapping checks)
    Kingdee {
        #[command(subcommand)]
        command: KingdeeCommands,
    },

    /// Operations diagnostics (health checks, DB integrity, tracing toggles)
    Ops {
        #[command(subcommand)]
        command: OpsCommands,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// Run database migrations
    Migrate {
        /// Dry run (show migrations without applying)
        #[arg(long)]
        dry_run: bool,
    },

    /// Backup database
    Backup {
        /// Output path for backup file
        #[arg(short, long)]
        output: String,
    },

    /// Restore database from backup
    Restore {
        /// Input path for backup file
        #[arg(short, long)]
        input: String,

        /// Confirm restoration (required)
        #[arg(long)]
        yes_really: bool,
    },

    /// Verify database integrity
    Verify,
}

#[derive(Subcommand)]
enum ImportCommands {
    /// Import items from CSV/JSON
    Items {
        /// Input file path
        #[arg(short, long)]
        file: String,

        /// Dry run (validate without importing)
        #[arg(long)]
        dry_run: bool,
    },

    /// Import locations from CSV/JSON
    Locations {
        /// Input file path
        #[arg(short, long)]
        file: String,

        /// Dry run (validate without importing)
        #[arg(long)]
        dry_run: bool,
    },

    /// Import suppliers from CSV/JSON
    Suppliers {
        /// Input file path
        #[arg(short, long)]
        file: String,

        /// Dry run (validate without importing)
        #[arg(long)]
        dry_run: bool,
    },

    /// Import customers from CSV/JSON
    Customers {
        /// Input file path
        #[arg(short, long)]
        file: String,

        /// Dry run (validate without importing)
        #[arg(long)]
        dry_run: bool,
    },

    /// Import initial stock from CSV/JSON
    InitialStock {
        /// Input file path
        #[arg(short, long)]
        file: String,

        /// Dry run (validate without importing)
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum ExportCommands {
    /// Export inventory snapshot
    InventorySnapshot {
        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Export format (csv, json)
        #[arg(short, long, default_value = "csv")]
        format: String,
    },

    /// Export audit extract
    AuditExtract {
        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,

        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
    },

    /// Export shipment manifest
    ShipmentManifest {
        /// Shipment ID
        #[arg(short, long)]
        shipment_id: String,

        /// Output file path
        #[arg(short, long)]
        output: String,
    },
}

#[derive(Subcommand)]
enum ComplianceCommands {
    /// Validate compliance gates
    Validate {
        /// Entity type (shipment, transfer, release)
        #[arg(short, long)]
        entity_type: String,

        /// Entity ID
        #[arg(short, long)]
        entity_id: String,
    },

    /// List missing compliance documents
    ListMissing {
        /// Entity type (shipment, transfer, release)
        #[arg(short, long)]
        entity_type: String,

        /// Entity ID
        #[arg(short, long)]
        entity_id: String,
    },

    /// Build compliance document pack
    BuildPack {
        /// Entity type (shipment, transfer, release)
        #[arg(short, long)]
        entity_type: String,

        /// Entity ID
        #[arg(short, long)]
        entity_id: String,

        /// Output directory
        #[arg(short, long)]
        output: String,
    },
}

#[derive(Subcommand)]
enum KingdeeCommands {
    /// Trigger Kingdee sync
    Sync {
        /// Entity type to sync
        #[arg(short, long)]
        entity_type: Option<String>,

        /// Dry run (show what would be synced)
        #[arg(long)]
        dry_run: bool,
    },

    /// Retry failed sync operations
    Retry {
        /// Sync operation ID
        #[arg(short, long)]
        operation_id: Option<String>,
    },

    /// Reconcile data with Kingdee
    Reconcile {
        /// Entity type to reconcile
        #[arg(short, long)]
        entity_type: String,

        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,

        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
    },

    /// Check mapping configuration
    CheckMappings,
}

#[derive(Subcommand)]
enum OpsCommands {
    /// Run health checks
    Health,

    /// Check database integrity
    DbIntegrity,

    /// Toggle tracing level
    Tracing {
        /// New tracing level (trace, debug, info, warn, error)
        #[arg(short, long)]
        level: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = if let Some(config_path) = &cli.config {
        CliConfig::load_from_file(config_path)?
    } else {
        CliConfig::load()?
    };

    // Initialize logging
    let log_level = cli.log_level.as_deref().unwrap_or(&config.log_level);
    init_logging(log_level);

    // Determine output format
    let output_format = if let Some(format) = &cli.format {
        OutputFormat::from_str(format)
            .map_err(|e| anyhow::anyhow!(e))?
    } else {
        OutputFormat::from_str(&config.output_format)
            .map_err(|e| anyhow::anyhow!(e))?
    };

    let writer = OutputWriter::new(output_format);

    tracing::info!("Starting inventory-cli");
    tracing::debug!("Configuration: {:?}", config);

    // Execute command
    match cli.command {
        Commands::Db { command } => handle_db_command(command, &config, &writer),
        Commands::Import { command } => handle_import_command(command, &config, &writer),
        Commands::Export { command } => handle_export_command(command, &config, &writer),
        Commands::Compliance { command } => handle_compliance_command(command, &config, &writer),
        Commands::Kingdee { command } => handle_kingdee_command(command, &config, &writer),
        Commands::Ops { command } => handle_ops_command(command, &config, &writer),
    }
}

#[tokio::main]
async fn handle_db_command(command: DbCommands, config: &CliConfig, writer: &OutputWriter) -> Result<()> {
    let db_path = &config.db_path;

    match command {
        DbCommands::Migrate { dry_run } => {
            let count = cli_db_migrations::run_migrations(db_path, dry_run).await?;

            if dry_run {
                writer.success(&format!("Dry run complete. {} migration(s) would be applied", count))?;
            } else {
                writer.success("Database migrations completed successfully")?;
            }
        }
        DbCommands::Backup { output } => {
            let backup_path = PathBuf::from(output);
            let bytes = cli_db_migrations::backup_database(db_path, &backup_path).await?;

            writer.success(&format!(
                "Database backed up successfully to {} ({} bytes)",
                backup_path.display(),
                bytes
            ))?;
        }
        DbCommands::Restore { input, yes_really } => {
            if !yes_really {
                writer.error("--yes-really flag is required for restore operations")?;
                return Err(anyhow::anyhow!("Confirmation required"));
            }

            let backup_path = PathBuf::from(input);
            let bytes = cli_db_migrations::restore_database(&backup_path, db_path).await?;

            writer.success(&format!(
                "Database restored successfully from {} ({} bytes)",
                backup_path.display(),
                bytes
            ))?;
        }
        DbCommands::Verify => {
            let result = cli_db_migrations::verify_database(db_path).await?;

            if writer.format() == OutputFormat::Json {
                writer.data(&result)?;
            } else {
                if result.passed {
                    writer.success("Database verification passed")?;
                } else {
                    writer.error("Database verification failed")?;
                }

                writer.info(&format!("File size: {} bytes", result.file_size))?;
                writer.info(&format!("Table count: {}", result.table_count))?;
                writer.info(&format!("Integrity check: {}", result.integrity_check))?;

                if !result.issues.is_empty() {
                    writer.warning(&format!("Issues found: {}", result.issues.len()))?;
                    for issue in &result.issues {
                        writer.info(&format!("  - {}", issue))?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_import_command(command: ImportCommands, _config: &CliConfig, writer: &OutputWriter) -> Result<()> {
    match command {
        ImportCommands::Items { file, dry_run } => {
            if dry_run {
                writer.info(&format!("Dry run: would import items from {}", file))?;
            } else {
                writer.info(&format!("Import items from {} not yet implemented", file))?;
            }
        }
        ImportCommands::Locations { file, dry_run } => {
            if dry_run {
                writer.info(&format!("Dry run: would import locations from {}", file))?;
            } else {
                writer.info(&format!("Import locations from {} not yet implemented", file))?;
            }
        }
        ImportCommands::Suppliers { file, dry_run } => {
            if dry_run {
                writer.info(&format!("Dry run: would import suppliers from {}", file))?;
            } else {
                writer.info(&format!("Import suppliers from {} not yet implemented", file))?;
            }
        }
        ImportCommands::Customers { file, dry_run } => {
            if dry_run {
                writer.info(&format!("Dry run: would import customers from {}", file))?;
            } else {
                writer.info(&format!("Import customers from {} not yet implemented", file))?;
            }
        }
        ImportCommands::InitialStock { file, dry_run } => {
            if dry_run {
                writer.info(&format!("Dry run: would import initial stock from {}", file))?;
            } else {
                writer.info(&format!("Import initial stock from {} not yet implemented", file))?;
            }
        }
    }
    Ok(())
}

fn handle_export_command(command: ExportCommands, _config: &CliConfig, writer: &OutputWriter) -> Result<()> {
    match command {
        ExportCommands::InventorySnapshot { output, format } => {
            writer.info(&format!("Export inventory snapshot to {} ({}) not yet implemented", output, format))?;
        }
        ExportCommands::AuditExtract { output, from, to } => {
            let date_range = match (from, to) {
                (Some(f), Some(t)) => format!(" from {} to {}", f, t),
                (Some(f), None) => format!(" from {}", f),
                (None, Some(t)) => format!(" to {}", t),
                (None, None) => String::new(),
            };
            writer.info(&format!("Export audit extract to {}{} not yet implemented", output, date_range))?;
        }
        ExportCommands::ShipmentManifest { shipment_id, output } => {
            writer.info(&format!("Export shipment manifest {} to {} not yet implemented", shipment_id, output))?;
        }
    }
    Ok(())
}

fn handle_compliance_command(command: ComplianceCommands, _config: &CliConfig, writer: &OutputWriter) -> Result<()> {
    match command {
        ComplianceCommands::Validate { entity_type, entity_id } => {
            writer.info(&format!("Validate compliance for {} {} not yet implemented", entity_type, entity_id))?;
        }
        ComplianceCommands::ListMissing { entity_type, entity_id } => {
            writer.info(&format!("List missing docs for {} {} not yet implemented", entity_type, entity_id))?;
        }
        ComplianceCommands::BuildPack { entity_type, entity_id, output } => {
            writer.info(&format!("Build compliance pack for {} {} to {} not yet implemented", entity_type, entity_id, output))?;
        }
    }
    Ok(())
}

fn handle_kingdee_command(command: KingdeeCommands, _config: &CliConfig, writer: &OutputWriter) -> Result<()> {
    match command {
        KingdeeCommands::Sync { entity_type, dry_run } => {
            let entity = entity_type.as_deref().unwrap_or("all");
            if dry_run {
                writer.info(&format!("Dry run: would sync {} to Kingdee", entity))?;
            } else {
                writer.info(&format!("Sync {} to Kingdee not yet implemented", entity))?;
            }
        }
        KingdeeCommands::Retry { operation_id } => {
            let op = operation_id.as_deref().unwrap_or("all failed");
            writer.info(&format!("Retry Kingdee sync {} not yet implemented", op))?;
        }
        KingdeeCommands::Reconcile { entity_type, from, to } => {
            let date_range = match (from, to) {
                (Some(f), Some(t)) => format!(" from {} to {}", f, t),
                (Some(f), None) => format!(" from {}", f),
                (None, Some(t)) => format!(" to {}", t),
                (None, None) => String::new(),
            };
            writer.info(&format!("Reconcile {} with Kingdee{} not yet implemented", entity_type, date_range))?;
        }
        KingdeeCommands::CheckMappings => {
            writer.info("Check Kingdee mappings not yet implemented")?;
        }
    }
    Ok(())
}

fn handle_ops_command(command: OpsCommands, _config: &CliConfig, writer: &OutputWriter) -> Result<()> {
    match command {
        OpsCommands::Health => {
            writer.info("Health checks not yet implemented")?;
        }
        OpsCommands::DbIntegrity => {
            writer.info("Database integrity checks not yet implemented")?;
        }
        OpsCommands::Tracing { level } => {
            writer.info(&format!("Toggle tracing to {} not yet implemented", level))?;
        }
    }
    Ok(())
}

