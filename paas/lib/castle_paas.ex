defmodule CastlePaaS do
  @moduledoc """
  Ash application/control plane for CASTLE.

  Ash owns domain actions and persistence, AshR2RML owns semantic correspondence,
  Reactor owns saga orchestration, ggen owns generated filesystem consequences, and
  the CASTLE kernel remains the exclusive admission/BRCE authority boundary.
  """

  @spec semantic_bundle() :: {:ok, map()} | {:error, term()}
  def semantic_bundle do
    CastlePaaS.Domain
    |> Ash.Domain.Info.resources()
    |> AshR2RML.Ggen.compile_ash_ttl_bundle()
  end

  @spec api_bundle(keyword()) :: {:ok, map()} | {:error, term()}
  def api_bundle(opts \\ [json_api: true]) do
    resources = Ash.Domain.Info.resources(CastlePaaS.Domain)

    with {:ok, bundle} <- AshR2RML.Compiler.compile_resources(resources),
         {:ok, ash_source} <- AshR2RML.Semantic.Ash.render(bundle, opts) do
      {:ok,
       %{
         status: :PARTIAL_ALIVE,
         standing: :construct_only,
         files: %{"generated/ash/api_resources.ex" => ash_source}
       }}
    end
  end

  @spec kernel() :: module()
  def kernel, do: Application.fetch_env!(:castle_paas, :kernel_module)
end

defmodule CastlePaaS.Repo do
  use AshPostgres.Repo, otp_app: :castle_paas

  @impl AshPostgres.Repo
  def installed_extensions, do: ["ash-functions"]

  @impl AshPostgres.Repo
  def min_pg_version, do: %Version{major: 14, minor: 0, patch: 0}
end

defmodule CastlePaaS.Application do
  use Application

  @impl true
  def start(_type, _args) do
    Supervisor.start_link(
      [CastlePaaS.Repo],
      strategy: :one_for_one,
      name: CastlePaaS.Supervisor
    )
  end
end
