terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }
}

provider "azurerm" {
  features {}
}

# ── Resource Group ─────────────────────────────────────────────────────────────

resource "azurerm_resource_group" "main" {
  name     = "favnir-crosscloud-demo-${var.env_suffix}"
  location = var.azure_location
}

# ── Azure DB for PostgreSQL Flexible Server ────────────────────────────────────

resource "azurerm_postgresql_flexible_server" "target" {
  name                   = "favnir-crosscloud-pg-${var.env_suffix}"
  resource_group_name    = azurerm_resource_group.main.name
  location               = azurerm_resource_group.main.location
  version                = "16"
  administrator_login    = "favnir"
  administrator_password = var.azure_pg_password
  sku_name               = "B_Standard_B1ms"
  storage_mb             = 32768
  backup_retention_days  = 7

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "azurerm_postgresql_flexible_server_firewall_rule" "allow_all" {
  name             = "allow-all-demo"
  server_id        = azurerm_postgresql_flexible_server.target.id
  start_ip_address = "0.0.0.0"
  end_ip_address   = "255.255.255.255"
}

resource "azurerm_postgresql_flexible_server_database" "appdb" {
  name      = "appdb"
  server_id = azurerm_postgresql_flexible_server.target.id
  collation = "en_US.utf8"
  charset   = "utf8"
}

# ── Storage Account + Blob Container (proof 証跡) ──────────────────────────────

resource "azurerm_storage_account" "proof" {
  name                     = "favnircrosscloud${var.env_suffix}"
  resource_group_name      = azurerm_resource_group.main.name
  location                 = azurerm_resource_group.main.location
  account_tier             = "Standard"
  account_replication_type = "LRS"

  tags = {
    Project = "favnir-crosscloud"
  }
}

resource "azurerm_storage_container" "proof" {
  name                  = "proof"
  storage_account_name  = azurerm_storage_account.proof.name
  container_access_type = "private"
}
