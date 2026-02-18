# --- Preset ---

# --- Managed PostgreSQL Cluster ---

resource "twc_database_cluster" "main" {
  name      = "${var.project_name}-pg"
  type      = "postgres16"
  preset_id = var.db_preset_id

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
    privileges  = ["SELECT", "INSERT", "UPDATE", "DELETE", "CREATE", "TRUNCATE", "REFERENCES"]
  }
}
