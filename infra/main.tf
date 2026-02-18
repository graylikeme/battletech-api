terraform {
  required_version = ">= 1.0"

  required_providers {
    twc = {
      source = "tf.timeweb.cloud/timeweb-cloud/timeweb-cloud"
    }
  }
}

provider "twc" {
  token = var.twc_token
}
