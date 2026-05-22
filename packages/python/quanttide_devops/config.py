
from dynaconf import Dynaconf


settings = Dynaconf(
    settings_files=['settings.toml'],
    environments=True,
    load_dotenv=True,
    envvar_prefix='DEVOPSSDK',
)
