"""
Integrated tests for Command Line
"""

from typer.testing import CliRunner
from coding_devops_sdk.config import settings as coding_settings

from quanttide_devops.cli import app
from quanttide_devops.config import settings

runner = CliRunner()


def test_app():
    coding_remote_url = f'https://{coding_settings.AUTH_USERNAME}:{coding_settings.AUTH_TOKEN}@e.coding.net/{coding_settings.TEAM}/{coding_settings.DEFAULT_PROJECT_NAME}/{coding_settings.DEFAULT_DEPOT_NAME}.git'
    github_remote_url = f'https://github.com/quanttide/{settings.GITHUB_DEFAULT_REPO_NAME}.git'

    result = runner.invoke(app, ["repo", "sync", coding_remote_url, github_remote_url])
    assert result.exit_code == 0

