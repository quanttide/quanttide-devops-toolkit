"""
Git repository provider
"""

from pydantic import BaseModel, SecretStr


class GitRepoProvider(BaseModel):
    name: str
    base_url: str
    username: str
    password: SecretStr


class GitHubRepoProvider(GitRepoProvider):
    pass


class CodingDevOpsRepoProvider(GitRepoProvider):
    pass
