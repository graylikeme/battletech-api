# --- S3 Storage ---

resource "twc_s3_bucket" "resources" {
  name      = "${var.project_name}-resources"
  type      = "public"
  preset_id = var.s3_preset_id
}

resource "twc_s3_bucket_subdomain" "resources" {
  bucket_id    = twc_s3_bucket.resources.id
  subdomain    = "resources.${var.domain}"
  release_cert = true
  depends_on   = [twc_dns_rr.resources]
}

resource "twc_s3_bucket_directory" "roster" {
  bucket_id = twc_s3_bucket.resources.id
  name      = "roster"
}

resource "twc_s3_bucket_directory" "roster_templates" {
  bucket_id  = twc_s3_bucket.resources.id
  name       = "roster/templates"
  depends_on = [twc_s3_bucket_directory.roster]
}

resource "twc_s3_bucket_directory" "roster_patterns" {
  bucket_id  = twc_s3_bucket.resources.id
  name       = "roster/patterns"
  depends_on = [twc_s3_bucket_directory.roster]
}
