"""
CLI endpoints
"""

import typer

from quanttide_devops.repositories.repo import app as repo_app

app = typer.Typer()
app.add_typer(repo_app, name="repo")


if __name__ == "__main__":
    app()
