from pydantic import BaseSettings


class Settings(BaseSettings):
    hosted_zone_domain_name: str
    sso_metadata_url: str
    cognito_domain_prefix: str

    @property
    def app_domain(self):
        """The domain where the frontend app is hosted."""
        return f"crates.{self.hosted_zone_domain_name}"

    @property
    def api_domain(self):
        """The domain where the API is served."""
        return f"api.{self.app_domain}"

    class Config:
        env_file = "infrastructure/.env"
        env_file_encoding = "utf-8"
