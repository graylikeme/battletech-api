# --- DNS ---

data "twc_dns_zone" "main" {
  name = var.domain
}

resource "twc_dns_rr" "api" {
  zone_id = data.twc_dns_zone.main.id
  name    = "api"
  type    = "A"
  value   = var.ingress_ip
}
