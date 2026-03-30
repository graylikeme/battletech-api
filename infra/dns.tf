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

resource "twc_dns_rr" "roster" {
  zone_id = data.twc_dns_zone.main.id
  name    = "roster"
  type    = "A"
  value   = var.ingress_ip
}

resource "twc_dns_rr" "resources" {
  zone_id = data.twc_dns_zone.main.id
  name    = "resources"
  type    = "CNAME"
  value   = "s3.twcstorage.ru"
}
