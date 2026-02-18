# --- Preset ---

data "twc_database_preset" "main" {
  location = var.region
  type     = "postgres"
  cpu      = var.db_preset_cpu
  disk     = var.db_preset_disk
}

# --- Managed PostgreSQL Cluster ---

resource "twc_database_cluster" "main" {
  name      = "${var.project_name}-pg"
  type      = "postgres"
  preset_id = data.twc_database_preset.main.id

  network {
    id = twc_vpc.main.id
  }
}

# --- Database ---

resource "twc_database_instance" "main" {
  cluster_id = twc_database_cluster.main.id
  name       = var.db_name
}

# --- Application User ---

resource "twc_database_user" "app" {
  cluster_id = twc_database_cluster.main.id
  login      = var.db_user
  password   = var.db_password

  instance {
    instance_id = twc_database_instance.main.id
    privileges  = ["SELECT", "INSERT", "UPDATE", "DELETE", "CREATE", "ALTER", "DROP", "REFERENCES", "INDEX"]
  }
}
