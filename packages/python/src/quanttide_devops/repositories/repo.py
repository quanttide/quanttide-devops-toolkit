"""
Git repository with multiple providers support
"""
import tempfile

from pydantic import BaseModel
from git import Repo
import typer

app = typer.Typer()


class GitRepo(BaseModel):
    remote_url: str
    provider: str


@app.command('sync')
def sync_repo(url_from: str, url_to: str):
    """

    :param url_from:
    :param url_to:
    """
    # 临时仓库
    tmp_dir = tempfile.TemporaryDirectory()
    # clone到临时仓库
    repo = Repo.clone_from(url_from, tmp_dir.name)
    # 增加remote并push
    new_origin = repo.create_remote('new_origin', url_to)
    new_origin.push()
    # 清空临时仓库
    tmp_dir.cleanup()
