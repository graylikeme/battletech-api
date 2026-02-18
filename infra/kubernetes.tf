# --- Presets ---

data "twc_k8s_preset" "master" {
  cpu      = var.k8s_master_cpu
  type     = "master"
  location = var.region
}

data "twc_k8s_preset" "worker" {
  cpu      = var.k8s_worker_cpu
  type     = "worker"
  location = var.region
}

# --- K8s Cluster ---

resource "twc_k8s_cluster" "main" {
  name              = "${var.project_name}-cluster"
  description       = "Managed Kubernetes cluster for ${var.project_name}"
  version           = var.k8s_version
  network_driver    = "flannel"
  high_availability = false
  ingress           = true
  preset_id         = data.twc_k8s_preset.master.id
  network_id        = twc_vpc.main.id
}

# --- Worker Node Group ---

resource "twc_k8s_node_group" "workers" {
  cluster_id = twc_k8s_cluster.main.id
  name       = "${var.project_name}-workers"
  node_count = var.k8s_worker_count
  preset_id  = data.twc_k8s_preset.worker.id
}
