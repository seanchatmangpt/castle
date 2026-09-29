defmodule CastlePaaS.MixProject do
  use Mix.Project

  @version "26.8.26"

  def project do
    [
      app: :castle_paas,
      version: @version,
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      elixirc_paths: ["lib", "generated/lib"],
      aliases: aliases()
    ]
  end

  def application do
    [
      mod: {CastlePaaS.Application, []},
      extra_applications: [:logger, :crypto]
    ]
  end

  defp deps do
    [
      {:ash, "~> 3.32.1"},
      {:ash_postgres, "~> 2.12.0"},
      {:ash_json_api, "~> 1.7.1"},
      {:ash_r2rml,
       github: "seanchatmangpt/ash_r2rml",
       ref: "067954ad406fd637fd47646bdb10c4580809c79d"},
      {:reactor, "~> 1.0.1"},
      {:jason, "~> 1.4"},
      {:plug_cowboy, "~> 2.7"},
      {:open_api_spex, "~> 3.16"}
    ]
  end

  defp aliases do
    [
      setup: ["deps.get"],
      verify: ["format --check-formatted", "compile --warnings-as-errors", "test"]
    ]
  end
end
