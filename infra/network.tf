# --- VPC ---

resource "twc_vpc" "main" {
  name        = "${var.project_name}-vpc"
  description = "Private network for ${var.project_name} K8s and DB"
  subnet_v4   = "10.0.0.0/16"
  location    = var.region
}

# --- Firewall for the database ---

resource "twc_firewall" "db" {
  name        = "${var.project_name}-db-firewall"
  description = "Restrict database access to VPC only"

  link {
    id   = twc_database_cluster.main.id
    type = "dbaas"
  }
}

# Allow PostgreSQL from the VPC subnet
resource "twc_firewall_rule" "db_allow_vpc" {
  firewall_id = twc_firewall.db.id
  description = "Allow PostgreSQL from VPC"
  direction   = "ingress"
  protocol    = "tcp"
  port        = 5432
  cidr        = twc_vpc.main.subnet_v4
}

# Drop all other inbound by default (TWC firewalls deny non-matching traffic)
