# --- Provider ---

variable "twc_token" {
  description = "Timeweb Cloud API token"
  type        = string
  sensitive   = true
}

# --- General ---

variable "region" {
  description = "Data center location"
  type        = string
  default     = "ru-1"
}

variable "project_name" {
  description = "Project name used for resource naming"
  type        = string
  default     = "battletech"
}

# --- Kubernetes ---

variable "k8s_version" {
  description = "Kubernetes version (check available versions in TWC console)"
  type        = string
  default     = "v1.30"
}

variable "k8s_master_cpu" {
  description = "CPU count for the K8s master node preset"
  type        = number
  default     = 2
}

variable "k8s_worker_cpu" {
  description = "CPU count for the K8s worker node preset"
  type        = number
  default     = 2
}

variable "k8s_worker_count" {
  description = "Number of worker nodes"
  type        = number
  default     = 1
}

# --- Database ---

variable "db_preset_cpu" {
  description = "CPU count for the database preset"
  type        = number
  default     = 1
}

variable "db_preset_disk" {
  description = "Disk size in MB for the database preset (e.g. 10240 = 10 GB)"
  type        = number
  default     = 10240
}

variable "db_name" {
  description = "PostgreSQL database name"
  type        = string
  default     = "battletech"
}

variable "db_user" {
  description = "PostgreSQL application user"
  type        = string
  default     = "battletech"
}

variable "db_password" {
  description = "PostgreSQL application user password"
  type        = string
  sensitive   = true
}

# --- Application ---

variable "api_image" {
  description = "Docker image for the BattleTech API"
  type        = string
  default     = "ghcr.io/graylikeme/battletech-api:latest"
}
