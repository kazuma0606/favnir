data "aws_ssm_parameter" "sap_base_url" {
  name            = "/favnir/sap/base_url"
  with_decryption = false
}

data "aws_ssm_parameter" "sap_username" {
  name            = "/favnir/sap/username"
  with_decryption = false
}

data "aws_ssm_parameter" "sap_password" {
  name            = "/favnir/sap/password"
  with_decryption = false
}

data "aws_ssm_parameter" "sap_client" {
  name            = "/favnir/sap/client"
  with_decryption = false
}
