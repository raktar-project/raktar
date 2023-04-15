from pydantic import BaseSettings


class Settings(BaseSettings):
    domain_name: str
    hosted_zone_domain_name: str

    class Config:
        env_file = "infrastructure/.env"
        env_file_encoding = "utf-8"
