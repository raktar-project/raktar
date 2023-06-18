from pydantic import BaseSettings


class Settings(BaseSettings):
    domain_name: str
    hosted_zone_domain_name: str
    sso_metadata_url: str
    cognito_domain_prefix: str

    class Config:
        env_file = "infrastructure/.env"
        env_file_encoding = "utf-8"
