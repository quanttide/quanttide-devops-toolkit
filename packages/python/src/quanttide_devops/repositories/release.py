"""
Domain model of Git release

GitHub: https://docs.github.com/en/rest/releases/releases?apiVersion=2022-11-28#create-a-release
GitLab: https://docs.gitlab.com/ee/api/releases/
"""

from pydantic import BaseModel


class GitRelease(BaseModel):
    tag_name: str
    title: str
    description: str
