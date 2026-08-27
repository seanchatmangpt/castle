[
  import_deps: [:ash, :ash_postgres, :ash_json_api, :ash_r2rml, :reactor],
  plugins: [Spark.Formatter],
  inputs: [
    "{mix,.formatter}.exs",
    "config/*.exs",
    "lib/**/*.{ex,exs}",
    "generated/lib/**/*.{ex,exs}",
    "test/**/*.{ex,exs}"
  ]
]
