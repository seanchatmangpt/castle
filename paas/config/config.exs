import Config

config :castle_paas,
  ash_domains: [CastlePaaS.Domain],
  ecto_repos: [CastlePaaS.Repo],
  kernel_module: CastlePaaS.Kernel.CLI

config :ash,
  redact_sensitive_values_in_errors?: true,
  transaction_rollback_on_error?: true

config :ash_json_api,
  use_deep_object_for_filter_type?: false

config :mime,
  extensions: %{"json" => "application/vnd.api+json"},
  types: %{"application/vnd.api+json" => ["json"]}

config :castle_paas, CastlePaaS.Repo,
  url: System.get_env("DATABASE_URL", "postgres://postgres:postgres@localhost/castle_paas_dev"),
  pool_size: String.to_integer(System.get_env("POOL_SIZE", "10")),
  show_sensitive_data_on_connection_error: false
