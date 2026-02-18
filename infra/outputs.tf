# --- Kubernetes ---

output "k8s_cluster_id" {
  description = "Managed K8s cluster ID"
  value       = twc_k8s_cluster.main.id
}

output "k8s_cluster_status" {
  description = "Current cluster status"
  value       = twc_k8s_cluster.main.status
}

output "kubeconfig" {
  description = "Kubeconfig for kubectl access"
  value       = twc_k8s_cluster.main.kubeconfig
  sensitive   = true
}

# --- Database ---

output "db_cluster_id" {
  description = "Managed PostgreSQL cluster ID"
  value       = twc_database_cluster.main.id
}

output "db_port" {
  description = "PostgreSQL port"
  value       = twc_database_cluster.main.port
}

output "db_host" {
  description = "PostgreSQL host (check TWC console for the VPC-internal IP)"
  value       = twc_database_cluster.main.networks
}

# --- DNS ---

output "api_dns_record" {
  description = "DNS A record for api subdomain"
  value       = "${twc_dns_rr.api.name}.${var.domain} → ${twc_dns_rr.api.value}"
}

# --- Network ---

output "vpc_id" {
  description = "VPC network ID"
  value       = twc_vpc.main.id
}

output "firewall_id" {
  description = "Database firewall ID"
  value       = twc_firewall.db.id
}
